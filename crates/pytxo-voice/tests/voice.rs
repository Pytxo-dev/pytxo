use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use pytxo_voice::{
    normalize_pcm_i16, pcm_i16_to_f32, trim_silence, ModelManager, PcmBuffer, PcmNormalizer,
    Transcriber, TranscriptSegment, VoiceError, VoiceSession, VoiceState,
};

#[test]
fn normalizes_stereo_48khz_capture_to_mono_16khz() {
    let stereo: Vec<i16> = (0..480).flat_map(|sample| [sample, sample]).collect();
    let normalized = normalize_pcm_i16(&stereo, 48_000, 2, 16_000);
    assert_eq!(normalized.len(), 160);
    assert_eq!(normalized[0], 0);
    assert_eq!(normalized[1], 3);
    assert_eq!(normalized[159], 477);
}

#[test]
fn streaming_normalizer_preserves_44khz_phase_across_callbacks() {
    let stereo: Vec<i16> = (0..4_410).flat_map(|sample| [sample, sample]).collect();
    let mut normalizer = PcmNormalizer::new(44_100, 2, 16_000);
    let mut streamed = Vec::new();
    for chunk in stereo.chunks(882) {
        streamed.extend(normalizer.push(chunk));
    }
    assert!((streamed.len() as isize - 1_600).abs() <= 1);
    assert!(streamed.windows(2).all(|pair| pair[1] >= pair[0]));
}

#[test]
fn session_enforces_capture_state_machine_and_cleanup() {
    let mut session = VoiceSession::new("default", "en", 4);
    assert_eq!(session.state(), VoiceState::Idle);
    session.start().unwrap();
    session.push_pcm(&[1, 2, 3, 4]).unwrap();
    assert_eq!(session.buffered_samples(), 4);
    session.pause().unwrap();
    session.resume().unwrap();
    session.finish_capture().unwrap();
    assert_eq!(session.state(), VoiceState::Transcribing);
    session
        .complete(vec![TranscriptSegment::new("ship Flow", 0.91)])
        .unwrap();
    assert_eq!(session.state(), VoiceState::Ready);
    assert_eq!(session.buffered_samples(), 0);
}

#[test]
fn bounded_buffer_keeps_latest_samples() {
    let mut buffer = PcmBuffer::new(4);
    buffer.push(&[1, 2, 3]);
    buffer.push(&[4, 5, 6]);
    assert_eq!(buffer.to_vec(), vec![3, 4, 5, 6]);
    assert_eq!(buffer.dropped_samples(), 2);
}

#[test]
fn bounded_buffer_remains_correct_after_many_small_overflows() {
    let mut buffer = PcmBuffer::new(1024);
    for sample in 0..20_000_i16 {
        buffer.push(&[sample]);
    }
    let samples = buffer.to_vec();
    assert_eq!(samples.len(), 1024);
    assert_eq!(samples[0], 18_976);
    assert_eq!(samples[1023], 19_999);
}

struct StubTranscriber;

impl Transcriber for StubTranscriber {
    fn transcribe(&self, _: &[f32], _: &str) -> pytxo_voice::Result<Vec<TranscriptSegment>> {
        Ok(vec![TranscriptSegment::new("should not run", 1.0)])
    }
}

#[test]
fn cancellable_transcriber_fails_before_processing_when_cancelled() {
    let cancelled = Arc::new(AtomicBool::new(true));
    let result = StubTranscriber.transcribe_cancellable(&[0.1], "en", cancelled);
    assert!(matches!(result, Err(VoiceError::Cancelled)));
}

#[test]
fn cancellation_and_device_loss_zeroize_audio() {
    let mut cancelled = VoiceSession::new("default", "en", 8);
    cancelled.start().unwrap();
    cancelled.push_pcm(&[10, 20, 30]).unwrap();
    cancelled.cancel();
    assert_eq!(cancelled.state(), VoiceState::Cancelled);
    assert_eq!(cancelled.buffered_samples(), 0);

    let mut failed = VoiceSession::new("default", "en", 8);
    failed.start().unwrap();
    failed.push_pcm(&[10, 20, 30]).unwrap();
    failed.device_lost("microphone disconnected");
    assert_eq!(failed.state(), VoiceState::Failed);
    assert_eq!(failed.buffered_samples(), 0);
}

#[test]
fn converts_pcm_and_trims_leading_and_trailing_silence() {
    let converted = pcm_i16_to_f32(&[i16::MIN, 0, i16::MAX]);
    assert!((converted[0] + 1.0).abs() < 0.0001);
    assert_eq!(converted[1], 0.0);
    assert!(converted[2] > 0.99);
    assert_eq!(
        trim_silence(&[0.0, 0.01, 0.4, 0.3, 0.01], 0.05),
        &[0.4, 0.3]
    );
}

#[test]
fn model_manager_rejects_checksum_mismatch_and_uses_voice_directory() {
    let dir = tempfile::tempdir().unwrap();
    let manager = ModelManager::new(dir.path());
    let source = dir.path().join("download.part");
    fs::write(&source, b"not-a-model").unwrap();
    let err = manager
        .install_verified("base.en", &source, "00")
        .unwrap_err();
    assert!(err.to_string().contains("checksum"));
    assert!(!manager.model_path("base.en").exists());

    let checksum = ModelManager::sha256_file(&source).unwrap();
    let installed = manager
        .install_verified("base.en", &source, &checksum)
        .unwrap();
    assert_eq!(installed, dir.path().join("models/voice/base.en.bin"));
    assert!(installed.exists());
}

#[test]
fn default_model_manifest_is_pinned() {
    let model = pytxo_voice::default_voice_model();
    assert_eq!(model.id, "base.en");
    assert!(model
        .url
        .starts_with("https://huggingface.co/ggerganov/whisper.cpp/resolve/"));
    assert_eq!(model.sha256.len(), 64);
    assert!(!model.multilingual);
}

#[cfg(feature = "native-capture")]
#[test]
fn cpal_adapter_implements_capture_contract() {
    fn capture<T: pytxo_voice::AudioCapture>() {}
    capture::<pytxo_voice::CpalCapture>();
}

#[cfg(feature = "local-whisper")]
#[test]
fn whisper_adapter_implements_transcriber_contract() {
    fn transcriber<T: pytxo_voice::Transcriber>() {}
    transcriber::<pytxo_voice::WhisperTranscriber>();
}
