use std::fs;

use pytxo_voice::{
    pcm_i16_to_f32, trim_silence, ModelManager, PcmBuffer, TranscriptSegment, VoiceSession,
    VoiceState,
};

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
    assert_eq!(buffer.as_slice(), &[3, 4, 5, 6]);
    assert_eq!(buffer.dropped_samples(), 2);
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
