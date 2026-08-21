## 1. Project Scaffold

- [ ] 1.1 Create the initial Tauri 2 application scaffold with `src/index.html`, `src/main.ts`, `src/styles.css`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, and Tauri config.
- [ ] 1.2 Configure the app to use Vanilla TypeScript/HTML/CSS only and verify no frontend framework is added.
- [ ] 1.3 Add Rust modules matching the simple service layout: `app_state`, `services/audio`, `services/feedback`, `services/hotkey`, `services/settings`, `services/startup`, `services/text_output`, and `services/transcription`.
- [ ] 1.4 Add required dependencies for Tauri tray, global shortcut, autostart, `cpal`, `whisper-rs`, model download support, clipboard/text insertion, and Windows API access.

## 2. Application Shell And Settings

- [ ] 2.1 Configure startup so the application launches tray-first without opening a main dashboard window.
- [ ] 2.2 Implement tray menu items for Settings and Exit.
- [ ] 2.3 Implement the Settings window showing Start with Windows and the fixed shortcut `Ctrl + Alt + Space`.
- [ ] 2.4 Implement settings persistence for non-sensitive settings with Start with Windows defaulting to enabled.
- [ ] 2.5 Integrate `tauri-plugin-autostart` for user-level startup and surface autostart errors without crashing.

## 3. State Machine And Hotkey

- [ ] 3.1 Implement central Rust `DictationState` with `Idle`, `Recording`, `Transcribing`, `Cleaning`, and `Inserting`.
- [ ] 3.2 Implement a single `handle_hotkey()` flow that starts recording from `Idle`, stops recording from `Recording`, and ignores hotkey presses while processing.
- [ ] 3.3 Register the fixed global hotkey `Ctrl + Alt + Space` using `tauri-plugin-global-shortcut`.
- [ ] 3.4 Handle hotkey registration failure by keeping the app running and informing the user.
- [ ] 3.5 Verify no dictation queue or concurrent dictation path exists.

## 4. Audio Capture

- [ ] 4.1 Implement `AudioRecorder` with `cpal` using the default input device.
- [ ] 4.2 Start microphone capture only when entering `Recording` and stop/drop the stream when recording ends.
- [ ] 4.3 Store normal recording audio in memory and avoid temporary or persistent audio files.
- [ ] 4.4 Convert captured device samples to 16 kHz mono f32 PCM for Whisper input.
- [ ] 4.5 Handle missing microphone, unsupported sample format, and stream errors by returning safely to `Idle`.

## 5. Whisper Transcription

- [ ] 5.1 Implement model path resolution under the application local data `models` directory.
- [ ] 5.2 Implement first-run download for the default Whisper Small Multilingual model and skip redownload when present.
- [ ] 5.3 Initialize `whisper-rs` once at application startup or service startup and keep the loaded model in memory.
- [ ] 5.4 Run Whisper inference on a blocking worker/thread/task instead of the Tauri event loop.
- [ ] 5.5 Configure Whisper language as Turkish (`tr`) and return raw transcript without cleanup/rewrite.
- [ ] 5.6 Handle model download/init/inference failures without crashing and restore `Idle` state.
- [ ] 5.7 Handle empty or whitespace-only transcription by inserting no text and returning to `Idle`.

## 6. Text Output And Feedback

- [ ] 6.1 Implement Windows text output with clipboard write plus SendInput `Ctrl + V` behind `#[cfg(target_os = "windows")]`.
- [ ] 6.2 Preserve existing clipboard content on a best-effort basis and restore it after paste initiation where practical.
- [ ] 6.3 Preserve Turkish Unicode characters during clipboard insertion.
- [ ] 6.4 Add Linux development-only fallback for text output without treating Linux as product support.
- [ ] 6.5 Ensure insertion does not foreground the Settings window or otherwise steal focus from the target input.
- [ ] 6.6 Implement tray state feedback for Idle, Recording, and Processing.
- [ ] 6.7 Add short recording start and stop audio cues.

## 7. End-to-End Flow

- [ ] 7.1 Wire the offline flow as `Idle -> Recording -> Transcribing -> Inserting -> Idle` using the raw Whisper transcript.
- [ ] 7.2 Confirm `Cleaning` remains in the state model but no OpenCode Zen API request, AI cleanup service behavior, or OpenCode Go integration is implemented.
- [ ] 7.3 Confirm no audio history, transcript history, telemetry, database, local HTTP service, Python runtime, custom hotkey UI, microphone selection UI, model manager UI, frontend framework, realtime transcription, live subtitles, or floating rich UI is added.
- [ ] 7.4 Add safe user-facing error/status messages for microphone, hotkey, model, transcription, autostart, and insertion failures.

## 8. Verification

- [ ] 8.1 Run Rust formatting and lint/check commands available in the project.
- [ ] 8.2 Run frontend formatting/build/check commands available in the project.
- [ ] 8.3 On Linux development environment, verify the app compiles and Windows-only text output is guarded by conditional compilation.
- [ ] 8.4 On Windows 10/11 x64, verify tray startup, Settings opening, Exit, autostart behavior, and hotkey registration.
- [ ] 8.5 On Windows 10/11 x64, verify `Ctrl + Alt + Space -> speech -> Ctrl + Alt + Space -> local Whisper -> paste` works in at least Notepad and one Chromium-based browser input.
- [ ] 8.6 On Windows 10/11 x64, verify Turkish characters paste correctly and previous clipboard content is restored on a best-effort basis.
- [ ] 8.7 Verify no text is inserted for empty transcription and all failure paths return safely to `Idle`.
