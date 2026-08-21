use std::sync::Mutex;
use tauri::Manager;

use crate::services::{
    audio::AudioRecorder, feedback, settings::Settings, text_output, transcription,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DictationState {
    #[default]
    Idle,
    Recording,
    Transcribing,
    #[allow(dead_code)]
    Cleaning,
    Inserting,
}

pub struct AppState {
    pub dictation_state: Mutex<DictationState>,
    pub audio_recorder: Mutex<Option<AudioRecorder>>,
    pub settings: Mutex<Settings>,
    pub last_error: Mutex<Option<String>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        // Note: Clone for Managed State usage in hotkey thread - shares same Arc inner via Tauri state cloning via inner value
        // This is a shallow structural clone for setup; real state is shared via Tauri's managed state.
        // For hotkey registration we pass owned clone via Arc.
        Self {
            dictation_state: Mutex::new(self.dictation_state.lock().unwrap().clone()),
            audio_recorder: Mutex::new(None),
            settings: Mutex::new(self.settings.lock().unwrap().clone()),
            last_error: Mutex::new(self.last_error.lock().unwrap().clone()),
        }
    }
}

// We need AppState to be shareable via Arc for hotkey callback; wrap in Arc internally?
// Simpler: handle_hotkey uses AppHandle to fetch Tauri managed state.

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            dictation_state: Mutex::new(DictationState::Idle),
            audio_recorder: Mutex::new(None),
            settings: Mutex::new(settings),
            last_error: Mutex::new(None),
        }
    }

    /// Single hotkey handler: Idle -> Recording, Recording -> Transcribing -> Inserting -> Idle, Processing ignored
    pub fn handle_hotkey(app: &tauri::AppHandle) {
        let state: tauri::State<AppState> = app.state();
        let current = state.dictation_state.lock().unwrap().clone();
        match current {
            DictationState::Idle => {
                // Try to start recording
                let mut recorder_guard = state.audio_recorder.lock().unwrap();
                let mut recorder = crate::services::audio::AudioRecorder::new();
                match recorder.start() {
                    Ok(_) => {
                        *recorder_guard = Some(recorder);
                        *state.dictation_state.lock().unwrap() = DictationState::Recording;
                        *state.last_error.lock().unwrap() = None;
                        feedback::update_tray_for_state(app, DictationState::Recording);
                        feedback::play_start_cue();
                        println!("Recording started");
                    }
                    Err(e) => {
                        eprintln!("microphone start failed: {}", e);
                        *state.dictation_state.lock().unwrap() = DictationState::Idle;
                        *state.last_error.lock().unwrap() =
                            Some(format!("Microphone error: {}", e));
                        feedback::update_tray_for_state(app, DictationState::Idle);
                    }
                }
            }
            DictationState::Recording => {
                // Stop recording and start transcription flow
                let audio_buffer = {
                    let mut guard = state.audio_recorder.lock().unwrap();
                    if let Some(mut rec) = guard.take() {
                        rec.stop();
                        rec.take_buffer()
                    } else {
                        Vec::new()
                    }
                };
                *state.dictation_state.lock().unwrap() = DictationState::Transcribing;
                feedback::update_tray_for_state(app, DictationState::Transcribing);
                feedback::play_stop_cue();

                // Spawn blocking transcription + insertion on separate thread
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    Self::process_recording(app_handle, audio_buffer);
                });
            }
            DictationState::Transcribing | DictationState::Cleaning | DictationState::Inserting => {
                // Ignore new hotkey while processing, non-crashing feedback
                println!("hotkey ignored while processing: {:?}", current);
                // Update last_error as non-fatal status (optional)
                // Do not queue
            }
        }
    }

    fn process_recording(app: tauri::AppHandle, audio: Vec<f32>) {
        // Verification: no queue exists - single threaded processing via state machine

        // Normalize empty check? even empty audio should go through transcription handling safely
        let state: tauri::State<AppState> = app.state();

        // Check if audio empty - transcribe will handle
        // Run Whisper inference on blocking worker (already in thread)
        let transcript_result = {
            // Ensure model path and whisper context available
            // The init is done at startup; here we just transcribe
            transcription::transcribe_blocking(audio)
        };

        let transcript = match transcript_result {
            Ok(t) => t,
            Err(e) => {
                eprintln!("transcription failed: {}", e);
                *state.last_error.lock().unwrap() = Some(format!("Transcription failed: {}", e));
                *state.dictation_state.lock().unwrap() = DictationState::Idle;
                feedback::update_tray_for_state(&app, DictationState::Idle);
                return;
            }
        };

        // Handle empty / whitespace-only transcription
        if transcript.trim().is_empty() {
            println!("empty transcript, returning to Idle without insertion");
            *state.dictation_state.lock().unwrap() = DictationState::Idle;
            feedback::update_tray_for_state(&app, DictationState::Idle);
            return;
        }

        // Cleaning state remains but no AI cleanup in this change - skip to Inserting
        // Explicitly keep Cleaning enum variant for spec compliance but don't enter remote work
        // If we wanted to show Cleaning state, we could briefly set it, but offline flow is Transcribing -> Inserting
        // So directly go to Inserting
        *state.dictation_state.lock().unwrap() = DictationState::Inserting;
        feedback::update_tray_for_state(&app, DictationState::Inserting);

        // Text output: clipboard + Ctrl+V on Windows, fallback on Linux
        match text_output::insert_text(&transcript) {
            Ok(_) => {
                println!("text inserted");
                *state.last_error.lock().unwrap() = None;
            }
            Err(e) => {
                eprintln!("text insertion failed: {}", e);
                *state.last_error.lock().unwrap() = Some(format!("Insertion failed: {}", e));
            }
        }

        *state.dictation_state.lock().unwrap() = DictationState::Idle;
        feedback::update_tray_for_state(&app, DictationState::Idle);
    }
}
