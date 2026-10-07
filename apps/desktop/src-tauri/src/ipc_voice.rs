use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use pytxo_voice::{
    default_voice_model, AudioCapture, CpalCapture, ModelManager, VoiceSession, VoiceState,
};
#[cfg(feature = "voice-whisper")]
use pytxo_voice::{pcm_i16_to_f32, Transcriber, WhisperTranscriber};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::ipc::AppState;
use crate::ipc_error::{map_lock_err, IpcResult, PytxoIpcError};

const MAX_CAPTURE_SAMPLES: usize = 16_000 * 60 * 5;

#[cfg(feature = "voice-whisper")]
struct SensitiveSamples(Vec<f32>);

#[cfg(feature = "voice-whisper")]
impl Drop for SensitiveSamples {
    fn drop(&mut self) {
        self.0.fill(0.0);
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct VoiceSessionDto {
    pub session_id: String,
    pub device: String,
    pub language: String,
    pub state: VoiceState,
    pub elapsed_ms: u64,
    pub buffered_samples: usize,
    pub confidence: Option<f32>,
    pub transcript_segments: Vec<pytxo_voice::TranscriptSegment>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[allow(dead_code)] // Transcription variants are emitted when the platform Whisper feature is enabled.
pub enum VoiceProgressEvent {
    AudioLevel {
        session_id: String,
        level: f32,
    },
    CaptureState {
        session_id: String,
        state: VoiceState,
    },
    TranscriptionProgress {
        session_id: String,
        progress: f32,
    },
    PartialTranscript {
        session_id: String,
        text: String,
        confidence: f32,
    },
}

fn emit_state(app: &AppHandle, session: &VoiceSessionDto) {
    let _ = app.emit(
        "pytxo://voice/progress",
        VoiceProgressEvent::CaptureState {
            session_id: session.session_id.clone(),
            state: session.state,
        },
    );
}

#[cfg(feature = "voice-whisper")]
fn fail_voice_session(
    state: &State<'_, AppState>,
    app: &AppHandle,
    id: Uuid,
    message: &str,
) -> IpcResult<()> {
    state
        .voice_captures
        .lock()
        .map_err(map_lock_err)?
        .remove(&id);
    state
        .voice_cancellations
        .lock()
        .map_err(map_lock_err)?
        .remove(&id);
    let failed = {
        let mut sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?;
        session.device_lost(message.to_string());
        dto(session)
    };
    emit_state(app, &failed);
    Ok(())
}

fn dto(session: &VoiceSession) -> VoiceSessionDto {
    VoiceSessionDto {
        session_id: session.session_id.to_string(),
        device: session.device.clone(),
        language: session.language.clone(),
        state: session.state(),
        elapsed_ms: session.elapsed_ms(),
        buffered_samples: session.buffered_samples(),
        confidence: session.confidence,
        transcript_segments: session.transcript_segments.clone(),
        error: session.error.clone(),
    }
}

fn parse_session_id(session_id: &str) -> IpcResult<Uuid> {
    Uuid::parse_str(session_id).map_err(|error| PytxoIpcError::from_err("voice_session", error))
}

#[tauri::command]
pub fn voice_list_devices() -> IpcResult<Vec<String>> {
    CpalCapture::default()
        .devices()
        .map_err(|error| PytxoIpcError::from_err("voice_device", error))
}

#[tauri::command]
pub fn voice_default_model() -> pytxo_voice::VoiceModel {
    default_voice_model()
}

#[tauri::command]
pub fn voice_local_available() -> bool {
    cfg!(feature = "voice-whisper")
}

#[tauri::command]
pub fn voice_model_status() -> IpcResult<Option<String>> {
    Ok(verified_default_model_path()?.map(|path| path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn voice_install_default_model() -> IpcResult<String> {
    let root = voice_data_root()?;
    tokio::task::spawn_blocking(move || {
        let model = default_voice_model();
        let download_dir = root.join("models").join("voice");
        std::fs::create_dir_all(&download_dir)
            .map_err(|error| PytxoIpcError::from_err("voice_model", error))?;
        let download = download_dir.join("base.en.download");
        let result = (|| -> IpcResult<String> {
            let response = ureq::get(&model.url)
                .call()
                .map_err(|error| PytxoIpcError::from_err("voice_model_download", error))?;
            let mut reader = response.into_reader();
            let mut file = std::fs::File::create(&download)
                .map_err(|error| PytxoIpcError::from_err("voice_model", error))?;
            std::io::copy(&mut reader, &mut file)
                .map_err(|error| PytxoIpcError::from_err("voice_model_download", error))?;
            ModelManager::new(&root)
                .install_verified(&model.id, &download, &model.sha256)
                .map(|path| path.to_string_lossy().into_owned())
                .map_err(|error| PytxoIpcError::from_err("voice_model", error))
        })();
        let _ = std::fs::remove_file(download);
        result
    })
    .await
    .map_err(|error| PytxoIpcError::from_err("voice_model", error))?
}

fn voice_data_root() -> IpcResult<std::path::PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .ok_or_else(|| PytxoIpcError::new("voice_model", "home directory unavailable"))?;
    Ok(home.join(".pytxo"))
}

fn verified_default_model_path() -> IpcResult<Option<std::path::PathBuf>> {
    let root = voice_data_root()?;
    let model = default_voice_model();
    let path = ModelManager::new(root).model_path(&model.id);
    if !path.exists() {
        return Ok(None);
    }
    let actual = ModelManager::sha256_file(&path)
        .map_err(|error| PytxoIpcError::from_err("voice_model", error))?;
    if !actual.eq_ignore_ascii_case(&model.sha256) {
        return Err(PytxoIpcError::new(
            "voice_model",
            "installed Voice model checksum mismatch",
        ));
    }
    Ok(Some(path))
}

#[tauri::command]
pub fn voice_start_session(
    device: String,
    language: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> IpcResult<VoiceSessionDto> {
    let _upgrade_guard = pytxo_core::UpgradeGuard::work()
        .map_err(|error| PytxoIpcError::from_err("update_busy", error))?;
    let mut session = VoiceSession::new(device.clone(), language, MAX_CAPTURE_SAMPLES);
    session
        .start()
        .map_err(|error| PytxoIpcError::from_err("voice_state", error))?;
    let id = session.session_id;
    {
        let mut sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
        if sessions.values().any(|session| {
            matches!(
                session.state(),
                VoiceState::Recording | VoiceState::Paused | VoiceState::Transcribing
            )
        }) {
            return Err(PytxoIpcError::new(
                "voice_session",
                "another Voice session is already active",
            ));
        }
        sessions.insert(id, session);
    }
    state
        .voice_cancellations
        .lock()
        .map_err(map_lock_err)?
        .insert(id, Arc::new(AtomicBool::new(false)));
    let sessions = Arc::clone(&state.voice_sessions);
    let event_app = app.clone();
    let sink = Arc::new(move |samples: &[i16]| {
        let accepted = if let Ok(mut sessions) = sessions.lock() {
            if let Some(session) = sessions.get_mut(&id) {
                session.push_pcm(samples).is_ok()
            } else {
                false
            }
        } else {
            false
        };
        if !accepted {
            return;
        }
        let peak = samples
            .iter()
            .map(|sample| sample.unsigned_abs())
            .max()
            .unwrap_or(0) as f32
            / i16::MAX as f32;
        let _ = event_app.emit(
            "pytxo://voice/progress",
            VoiceProgressEvent::AudioLevel {
                session_id: id.to_string(),
                level: peak.clamp(0.0, 1.0),
            },
        );
    });
    let error_sessions = Arc::clone(&state.voice_sessions);
    let error_captures = Arc::clone(&state.voice_captures);
    let error_app = app.clone();
    let error_sink = Arc::new(move |message: String| {
        let failed = error_sessions.lock().ok().and_then(|mut sessions| {
            sessions.get_mut(&id).map(|session| {
                session.device_lost(message.clone());
                dto(session)
            })
        });
        if let Some(session) = failed {
            emit_state(&error_app, &session);
        }
        let captures = Arc::clone(&error_captures);
        std::thread::spawn(move || {
            if let Ok(mut captures) = captures.lock() {
                captures.remove(&id);
            }
        });
    });
    let mut capture = CpalCapture::with_sink_and_error(sink, error_sink);
    if let Err(error) = capture.start(&device) {
        if let Ok(mut sessions) = state.voice_sessions.lock() {
            if let Some(session) = sessions.get_mut(&id) {
                session.device_lost(error.to_string());
            }
        }
        if let Ok(mut cancellations) = state.voice_cancellations.lock() {
            cancellations.remove(&id);
        }
        return Err(PytxoIpcError::from_err("voice_device", error));
    }
    state
        .voice_captures
        .lock()
        .map_err(map_lock_err)?
        .insert(id, capture);
    let failed_during_start = state
        .voice_sessions
        .lock()
        .map_err(map_lock_err)?
        .get(&id)
        .is_some_and(|session| session.state() == VoiceState::Failed);
    if failed_during_start {
        state
            .voice_captures
            .lock()
            .map_err(map_lock_err)?
            .remove(&id);
    }
    let session = voice_get_session(id.to_string(), state)?;
    emit_state(&app, &session);
    Ok(session)
}

#[tauri::command]
pub fn voice_pause_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> IpcResult<VoiceSessionDto> {
    let id = parse_session_id(&session_id)?;
    state
        .voice_captures
        .lock()
        .map_err(map_lock_err)?
        .get_mut(&id)
        .ok_or_else(|| PytxoIpcError::new("voice_session", "capture not found"))?
        .pause()
        .map_err(|error| PytxoIpcError::from_err("voice_device", error))?;
    state
        .voice_sessions
        .lock()
        .map_err(map_lock_err)?
        .get_mut(&id)
        .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?
        .pause()
        .map_err(|error| PytxoIpcError::from_err("voice_state", error))?;
    let session = voice_get_session(session_id, state)?;
    emit_state(&app, &session);
    Ok(session)
}

#[tauri::command]
pub fn voice_resume_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> IpcResult<VoiceSessionDto> {
    let _upgrade_guard = pytxo_core::UpgradeGuard::work()
        .map_err(|error| PytxoIpcError::from_err("update_busy", error))?;
    let id = parse_session_id(&session_id)?;
    state
        .voice_captures
        .lock()
        .map_err(map_lock_err)?
        .get_mut(&id)
        .ok_or_else(|| PytxoIpcError::new("voice_session", "capture not found"))?
        .resume()
        .map_err(|error| PytxoIpcError::from_err("voice_device", error))?;
    state
        .voice_sessions
        .lock()
        .map_err(map_lock_err)?
        .get_mut(&id)
        .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?
        .resume()
        .map_err(|error| PytxoIpcError::from_err("voice_state", error))?;
    let session = voice_get_session(session_id, state)?;
    emit_state(&app, &session);
    Ok(session)
}

#[tauri::command]
pub async fn voice_finish_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> IpcResult<VoiceSessionDto> {
    let _upgrade_guard = pytxo_core::UpgradeGuard::work()
        .map_err(|error| PytxoIpcError::from_err("update_busy", error))?;
    let id = parse_session_id(&session_id)?;
    if let Some(mut capture) = state
        .voice_captures
        .lock()
        .map_err(map_lock_err)?
        .remove(&id)
    {
        capture
            .stop()
            .map_err(|error| PytxoIpcError::from_err("voice_device", error))?;
    }
    {
        let mut sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?;
        session
            .finish_capture()
            .map_err(|error| PytxoIpcError::from_err("voice_state", error))?;
        emit_state(&app, &dto(session));
    }
    #[cfg(feature = "voice-whisper")]
    {
        let prepared = (|| -> IpcResult<_> {
            let model_path = verified_default_model_path()?.ok_or_else(|| {
                PytxoIpcError::new("voice_model", "install the verified base.en model first")
            })?;
            let cancelled = state
                .voice_cancellations
                .lock()
                .map_err(map_lock_err)?
                .get(&id)
                .cloned()
                .ok_or_else(|| PytxoIpcError::new("voice_session", "cancellation state missing"))?;
            let (samples, language) = {
                let sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
                let session = sessions
                    .get(&id)
                    .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?;
                let mut pcm = session
                    .samples_for_transcription()
                    .map_err(|error| PytxoIpcError::from_err("voice_state", error))?;
                let samples = pcm_i16_to_f32(&pcm);
                pcm.fill(0);
                (samples, session.language.clone())
            };
            Ok((samples, language, model_path, cancelled))
        })();
        let (samples, language, model_path, cancelled) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                fail_voice_session(&state, &app, id, &error.message)?;
                return Err(error);
            }
        };
        let _ = app.emit(
            "pytxo://voice/progress",
            VoiceProgressEvent::TranscriptionProgress {
                session_id: id.to_string(),
                progress: 0.1,
            },
        );
        let worker_cancelled = Arc::clone(&cancelled);
        let transcription_task = tokio::task::spawn_blocking(move || {
            let samples = SensitiveSamples(samples);
            let transcriber =
                WhisperTranscriber::new(model_path).map_err(|error| error.to_string())?;
            transcriber
                .transcribe_cancellable(&samples.0, &language, worker_cancelled)
                .map_err(|error| error.to_string())
        });
        let transcription = match transcription_task.await {
            Ok(result) => result,
            Err(error) => {
                state
                    .voice_cancellations
                    .lock()
                    .map_err(map_lock_err)?
                    .remove(&id);
                let message = format!("transcription worker failed: {error}");
                let failed = {
                    let mut sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
                    let session = sessions
                        .get_mut(&id)
                        .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?;
                    session.device_lost(message.clone());
                    dto(session)
                };
                emit_state(&app, &failed);
                return Err(PytxoIpcError::new("voice_transcription", message));
            }
        };
        state
            .voice_cancellations
            .lock()
            .map_err(map_lock_err)?
            .remove(&id);
        if !cancelled.load(Ordering::Acquire) {
            if let Ok(segments) = &transcription {
                for segment in segments {
                    let _ = app.emit(
                        "pytxo://voice/progress",
                        VoiceProgressEvent::PartialTranscript {
                            session_id: id.to_string(),
                            text: segment.text.clone(),
                            confidence: segment.confidence,
                        },
                    );
                }
                let _ = app.emit(
                    "pytxo://voice/progress",
                    VoiceProgressEvent::TranscriptionProgress {
                        session_id: id.to_string(),
                        progress: 1.0,
                    },
                );
            }
        }
        let mut sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?;
        if session.state() == VoiceState::Cancelled {
            return Ok(dto(session));
        }
        match transcription {
            Ok(segments) => session
                .complete(segments)
                .map_err(|error| PytxoIpcError::from_err("voice_state", error))?,
            Err(error) => {
                drop(sessions);
                fail_voice_session(&state, &app, id, &error)?;
                return Err(PytxoIpcError::new("voice_transcription", error));
            }
        }
    }
    #[cfg(not(feature = "voice-whisper"))]
    {
        state
            .voice_cancellations
            .lock()
            .map_err(map_lock_err)?
            .remove(&id);
        state
            .voice_sessions
            .lock()
            .map_err(map_lock_err)?
            .get_mut(&id)
            .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?
            .device_lost("local transcription support is not installed");
    }
    let session = voice_get_session(session_id, state)?;
    emit_state(&app, &session);
    Ok(session)
}

#[tauri::command]
pub fn voice_cancel_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> IpcResult<VoiceSessionDto> {
    let id = parse_session_id(&session_id)?;
    if let Some(cancelled) = state
        .voice_cancellations
        .lock()
        .map_err(map_lock_err)?
        .get(&id)
    {
        cancelled.store(true, Ordering::Release);
    }
    state
        .voice_captures
        .lock()
        .map_err(map_lock_err)?
        .remove(&id);
    let was_transcribing = {
        let mut sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))?;
        let was_transcribing = session.state() == VoiceState::Transcribing;
        session.cancel();
        was_transcribing
    };
    if !was_transcribing {
        state
            .voice_cancellations
            .lock()
            .map_err(map_lock_err)?
            .remove(&id);
    }
    let session = voice_get_session(session_id, state)?;
    emit_state(&app, &session);
    Ok(session)
}

#[tauri::command]
pub fn voice_get_session(
    session_id: String,
    state: State<'_, AppState>,
) -> IpcResult<VoiceSessionDto> {
    let id = parse_session_id(&session_id)?;
    let sessions = state.voice_sessions.lock().map_err(map_lock_err)?;
    sessions
        .get(&id)
        .map(dto)
        .ok_or_else(|| PytxoIpcError::new("voice_session", "session not found"))
}
