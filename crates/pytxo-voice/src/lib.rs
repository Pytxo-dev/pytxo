//! Local-first voice contracts for Pytxo Flow.
//!
//! Raw PCM is owned by a bounded in-memory buffer and is cleared on every terminal transition.
//! Persistence belongs to model files and sanitized transcripts only; this crate has no audio
//! recording format or recording-storage API.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("invalid voice transition from {from:?} to {to:?}")]
    InvalidTransition { from: VoiceState, to: VoiceState },
    #[error("audio capture is not recording")]
    NotRecording,
    #[error("model checksum mismatch for {model_id}")]
    ChecksumMismatch { model_id: String },
    #[error("invalid model id")]
    InvalidModelId,
    #[error("voice I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("voice backend failed: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, VoiceError>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceState {
    Idle,
    Recording,
    Paused,
    Transcribing,
    Ready,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TranscriptSegment {
    pub text: String,
    pub confidence: f32,
    pub uncertain: bool,
}

impl TranscriptSegment {
    pub fn new(text: impl Into<String>, confidence: f32) -> Self {
        let confidence = confidence.clamp(0.0, 1.0);
        Self {
            text: text.into(),
            confidence,
            uncertain: confidence < 0.75,
        }
    }
}

#[derive(Debug)]
pub struct PcmBuffer {
    samples: Vec<i16>,
    capacity: usize,
    dropped_samples: u64,
}

impl PcmBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
            dropped_samples: 0,
        }
    }

    pub fn push(&mut self, incoming: &[i16]) {
        if self.capacity == 0 {
            self.dropped_samples = self.dropped_samples.saturating_add(incoming.len() as u64);
            return;
        }
        let keep_from = incoming.len().saturating_sub(self.capacity);
        self.dropped_samples = self.dropped_samples.saturating_add(keep_from as u64);
        let incoming = &incoming[keep_from..];
        let overflow = self
            .samples
            .len()
            .saturating_add(incoming.len())
            .saturating_sub(self.capacity);
        if overflow > 0 {
            self.samples.drain(..overflow);
            self.dropped_samples = self.dropped_samples.saturating_add(overflow as u64);
        }
        self.samples.extend_from_slice(incoming);
    }

    pub fn as_slice(&self) -> &[i16] {
        &self.samples
    }

    pub fn dropped_samples(&self) -> u64 {
        self.dropped_samples
    }

    fn clear_sensitive(&mut self) {
        self.samples.fill(0);
        self.samples.clear();
    }
}

impl Drop for PcmBuffer {
    fn drop(&mut self) {
        self.clear_sensitive();
    }
}

#[derive(Debug)]
pub struct VoiceSession {
    pub session_id: Uuid,
    pub device: String,
    pub language: String,
    state: VoiceState,
    started_at: Option<Instant>,
    elapsed: Duration,
    buffer: PcmBuffer,
    pub transcript_segments: Vec<TranscriptSegment>,
    pub confidence: Option<f32>,
    pub error: Option<String>,
}

impl VoiceSession {
    pub fn new(device: impl Into<String>, language: impl Into<String>, max_samples: usize) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            device: device.into(),
            language: language.into(),
            state: VoiceState::Idle,
            started_at: None,
            elapsed: Duration::ZERO,
            buffer: PcmBuffer::new(max_samples),
            transcript_segments: Vec::new(),
            confidence: None,
            error: None,
        }
    }

    pub fn state(&self) -> VoiceState {
        self.state
    }

    pub fn elapsed_ms(&self) -> u64 {
        let live = self.started_at.map(|at| at.elapsed()).unwrap_or_default();
        self.elapsed.saturating_add(live).as_millis() as u64
    }

    pub fn buffered_samples(&self) -> usize {
        self.buffer.as_slice().len()
    }

    pub fn start(&mut self) -> Result<()> {
        self.transition(VoiceState::Idle, VoiceState::Recording)?;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    pub fn push_pcm(&mut self, samples: &[i16]) -> Result<()> {
        if self.state != VoiceState::Recording {
            return Err(VoiceError::NotRecording);
        }
        self.buffer.push(samples);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.transition(VoiceState::Recording, VoiceState::Paused)?;
        self.stop_clock();
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.transition(VoiceState::Paused, VoiceState::Recording)?;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    pub fn finish_capture(&mut self) -> Result<()> {
        if !matches!(self.state, VoiceState::Recording | VoiceState::Paused) {
            return Err(VoiceError::InvalidTransition {
                from: self.state,
                to: VoiceState::Transcribing,
            });
        }
        self.stop_clock();
        self.state = VoiceState::Transcribing;
        Ok(())
    }

    pub fn samples_for_transcription(&self) -> Result<&[i16]> {
        if self.state != VoiceState::Transcribing {
            return Err(VoiceError::InvalidTransition {
                from: self.state,
                to: VoiceState::Transcribing,
            });
        }
        Ok(self.buffer.as_slice())
    }

    pub fn complete(&mut self, segments: Vec<TranscriptSegment>) -> Result<()> {
        self.transition(VoiceState::Transcribing, VoiceState::Ready)?;
        self.confidence = (!segments.is_empty()).then(|| {
            segments
                .iter()
                .map(|segment| segment.confidence)
                .sum::<f32>()
                / segments.len() as f32
        });
        self.transcript_segments = segments;
        self.buffer.clear_sensitive();
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.stop_clock();
        self.state = VoiceState::Cancelled;
        self.buffer.clear_sensitive();
    }

    pub fn device_lost(&mut self, message: impl Into<String>) {
        self.stop_clock();
        self.state = VoiceState::Failed;
        self.error = Some(message.into());
        self.buffer.clear_sensitive();
    }

    fn transition(&mut self, expected: VoiceState, next: VoiceState) -> Result<()> {
        if self.state != expected {
            return Err(VoiceError::InvalidTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }

    fn stop_clock(&mut self) {
        if let Some(started_at) = self.started_at.take() {
            self.elapsed = self.elapsed.saturating_add(started_at.elapsed());
        }
    }
}

impl Drop for VoiceSession {
    fn drop(&mut self) {
        self.buffer.clear_sensitive();
    }
}

pub fn pcm_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|sample| *sample as f32 / 32_768.0)
        .collect()
}

pub fn trim_silence(samples: &[f32], threshold: f32) -> &[f32] {
    let start = samples
        .iter()
        .position(|sample| sample.abs() >= threshold)
        .unwrap_or(samples.len());
    let end = samples
        .iter()
        .rposition(|sample| sample.abs() >= threshold)
        .map(|index| index + 1)
        .unwrap_or(start);
    &samples[start..end]
}

pub trait AudioCapture: Send {
    fn devices(&self) -> Result<Vec<String>>;
    fn start(&mut self, device: &str) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

#[cfg(feature = "native-capture")]
type SampleSink = std::sync::Arc<dyn Fn(&[i16]) + Send + Sync>;

#[cfg(feature = "native-capture")]
pub struct CpalCapture {
    stream: Option<cpal::Stream>,
    sink: SampleSink,
}

#[cfg(feature = "native-capture")]
impl Default for CpalCapture {
    fn default() -> Self {
        Self::with_sink(std::sync::Arc::new(|_| {}))
    }
}

#[cfg(feature = "native-capture")]
impl CpalCapture {
    pub fn with_sink(sink: SampleSink) -> Self {
        Self { stream: None, sink }
    }
}

#[cfg(feature = "native-capture")]
impl AudioCapture for CpalCapture {
    fn devices(&self) -> Result<Vec<String>> {
        use cpal::traits::{DeviceTrait, HostTrait};

        cpal::default_host()
            .input_devices()
            .map_err(|error| VoiceError::Backend(error.to_string()))?
            .map(|device| {
                device
                    .description()
                    .map(|description| description.name().to_string())
                    .map_err(|error| VoiceError::Backend(error.to_string()))
            })
            .collect()
    }

    fn start(&mut self, requested_device: &str) -> Result<()> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();
        let device = if requested_device.is_empty() || requested_device == "default" {
            host.default_input_device()
        } else {
            host.input_devices()
                .map_err(|error| VoiceError::Backend(error.to_string()))?
                .find(|device| {
                    device
                        .description()
                        .is_ok_and(|description| description.name() == requested_device)
                })
        }
        .ok_or_else(|| VoiceError::Backend("input device unavailable".into()))?;
        let supported = device
            .default_input_config()
            .map_err(|error| VoiceError::Backend(error.to_string()))?;
        let sample_format = supported.sample_format();
        let config = supported.config();
        let error_callback = |error: cpal::StreamError| {
            eprintln!("Pytxo Voice capture error: {error}");
        };
        let stream = match sample_format {
            cpal::SampleFormat::I16 => {
                let sink = self.sink.clone();
                device.build_input_stream(
                    &config,
                    move |samples: &[i16], _| sink(samples),
                    error_callback,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let sink = self.sink.clone();
                device.build_input_stream(
                    &config,
                    move |samples: &[u16], _| {
                        let converted: Vec<i16> = samples
                            .iter()
                            .map(|sample| (*sample as i32 - 32_768) as i16)
                            .collect();
                        sink(&converted);
                    },
                    error_callback,
                    None,
                )
            }
            cpal::SampleFormat::F32 => {
                let sink = self.sink.clone();
                device.build_input_stream(
                    &config,
                    move |samples: &[f32], _| {
                        let converted: Vec<i16> = samples
                            .iter()
                            .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                            .collect();
                        sink(&converted);
                    },
                    error_callback,
                    None,
                )
            }
            other => {
                return Err(VoiceError::Backend(format!(
                    "unsupported input sample format: {other}"
                )))
            }
        }
        .map_err(|error| VoiceError::Backend(error.to_string()))?;
        stream
            .play()
            .map_err(|error| VoiceError::Backend(error.to_string()))?;
        self.stream = Some(stream);
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        use cpal::traits::StreamTrait;
        self.stream
            .as_ref()
            .ok_or(VoiceError::NotRecording)?
            .pause()
            .map_err(|error| VoiceError::Backend(error.to_string()))
    }

    fn resume(&mut self) -> Result<()> {
        use cpal::traits::StreamTrait;
        self.stream
            .as_ref()
            .ok_or(VoiceError::NotRecording)?
            .play()
            .map_err(|error| VoiceError::Backend(error.to_string()))
    }

    fn stop(&mut self) -> Result<()> {
        let _stream = self.stream.take().ok_or(VoiceError::NotRecording)?;
        Ok(())
    }
}

pub trait Transcriber: Send + Sync {
    fn transcribe(&self, pcm_16khz_mono: &[f32], language: &str) -> Result<Vec<TranscriptSegment>>;
}

#[cfg(feature = "local-whisper")]
pub struct WhisperTranscriber {
    context: whisper_rs::WhisperContext,
}

#[cfg(feature = "local-whisper")]
impl WhisperTranscriber {
    pub fn new(model_path: impl AsRef<Path>) -> Result<Self> {
        let context = whisper_rs::WhisperContext::new_with_params(
            model_path,
            whisper_rs::WhisperContextParameters::default(),
        )
        .map_err(|error| VoiceError::Backend(error.to_string()))?;
        Ok(Self { context })
    }
}

#[cfg(feature = "local-whisper")]
impl Transcriber for WhisperTranscriber {
    fn transcribe(&self, pcm_16khz_mono: &[f32], language: &str) -> Result<Vec<TranscriptSegment>> {
        use whisper_rs::{FullParams, SamplingStrategy};

        let samples = trim_silence(pcm_16khz_mono, 0.008);
        if samples.is_empty() {
            return Ok(Vec::new());
        }
        let mut state = self
            .context
            .create_state()
            .map_err(|error| VoiceError::Backend(error.to_string()))?;
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_language((!language.trim().is_empty()).then_some(language));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        state
            .full(params, samples)
            .map_err(|error| VoiceError::Backend(error.to_string()))?;
        state
            .as_iter()
            .map(|segment| {
                segment
                    .to_str_lossy()
                    .map(|text| TranscriptSegment::new(text.trim(), 0.85))
                    .map_err(|error| VoiceError::Backend(error.to_string()))
            })
            .filter(|segment| {
                segment
                    .as_ref()
                    .is_err_or(|segment| !segment.text.is_empty())
            })
            .collect()
    }
}

pub trait RemoteTranscriber: Send + Sync {
    fn provider_name(&self) -> &str;
    fn destination(&self) -> &str;
    fn retention_policy(&self) -> &str;
    fn transcribe_with_consent(
        &self,
        pcm_16khz_mono: &[f32],
        language: &str,
        session_consent: bool,
    ) -> Result<Vec<TranscriptSegment>>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VoiceModel {
    pub id: String,
    pub url: String,
    pub sha256: String,
    pub multilingual: bool,
}

pub fn default_voice_model() -> VoiceModel {
    VoiceModel {
        id: "base.en".into(),
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/80da2d8bfee42b0e836fc3a9890373e5defc00a6/ggml-base.en.bin".into(),
        sha256: "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002".into(),
        multilingual: false,
    }
}

#[derive(Clone, Debug)]
pub struct ModelManager {
    root: PathBuf,
}

impl ModelManager {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        Self {
            root: data_root.as_ref().join("models").join("voice"),
        }
    }

    pub fn model_path(&self, model_id: &str) -> PathBuf {
        self.root.join(format!("{model_id}.bin"))
    }

    pub fn sha256_file(path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut chunk = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut chunk)?;
            if read == 0 {
                break;
            }
            hasher.update(&chunk[..read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn install_verified(&self, model_id: &str, source: &Path, sha256: &str) -> Result<PathBuf> {
        if model_id.is_empty()
            || !model_id.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
            })
        {
            return Err(VoiceError::InvalidModelId);
        }
        let actual = Self::sha256_file(source)?;
        if !actual.eq_ignore_ascii_case(sha256) {
            return Err(VoiceError::ChecksumMismatch {
                model_id: model_id.to_string(),
            });
        }
        fs::create_dir_all(&self.root)?;
        let destination = self.model_path(model_id);
        let temporary = destination.with_extension("bin.part");
        let mut input = fs::File::open(source)?;
        let mut output = fs::File::create(&temporary)?;
        std::io::copy(&mut input, &mut output)?;
        output.flush()?;
        fs::rename(&temporary, &destination)?;
        Ok(destination)
    }
}
