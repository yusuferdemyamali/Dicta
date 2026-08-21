## Why

The product currently has a complete main specification but no implementation artifacts for the core offline Windows dictation flow. This change defines the first self-contained implementation slice: press `Ctrl + Alt + Space`, record speech, transcribe locally with Whisper, and paste the raw transcript into the active Windows text input without depending on AI cleanup.

## What Changes

- Add a tray-first Tauri 2 application shell with a minimal Settings window and tray menu entries for Settings and Exit.
- Add a central Rust-owned dictation state machine with `Idle`, `Recording`, `Transcribing`, `Cleaning`, and `Inserting` states, while this change uses the offline flow `Idle -> Recording -> Transcribing -> Inserting -> Idle`.
- Register the fixed global shortcut `Ctrl + Alt + Space` with toggle behavior using `tauri-plugin-global-shortcut`.
- Capture audio from the Windows default microphone with `cpal` only while recording, keep normal dictation audio in memory, and normalize captured samples to 16 kHz mono f32 PCM for Whisper.
- Add local Whisper transcription using `whisper-rs`/whisper.cpp with the default Small Multilingual model and Turkish language configuration.
- Store/download the default model in the application local data models directory and keep the loaded model in memory across dictations.
- Insert the raw Whisper transcript into the active Windows text input using clipboard plus `Ctrl + V` SendInput behavior, with best-effort clipboard preservation and Turkish Unicode support.
- Add minimal user feedback through tray state updates and short recording start/stop audio cues.
- Add autostart support using `tauri-plugin-autostart`, defaulting Start with Windows to enabled at user level.
- Keep OpenCode Zen / AI cleanup out of implementation; the `Cleaning` state remains part of the state model for mainspec alignment but performs no remote AI work in this change.

## Capabilities

### New Capabilities
- `local-windows-dictation`: Offline Windows dictation from fixed global hotkey through microphone capture, local Whisper transcription, and active text input insertion.

### Modified Capabilities

None.

## Impact

- Creates the initial Tauri 2 + Rust + Vanilla TypeScript/HTML/CSS application structure if it is not already present.
- Adds Rust modules aligned with the mainspec service structure: `app_state`, `services/audio`, `services/transcription`, `services/hotkey`, `services/text_output`, `services/settings`, `services/startup`, plus small Windows-specific code behind `#[cfg(target_os = "windows")]` where needed.
- Adds dependencies for Tauri tray/global shortcut/autostart, `cpal`, `whisper-rs`, model download support, clipboard/text insertion, and Windows native API calls.
- Adds minimal Settings UI for Start with Windows and fixed shortcut display; OpenCode Zen fields may exist only as inert settings structure if needed for mainspec-compatible UI layout.
- Does not add Python, a local HTTP server, database, history, telemetry, hotkey customization, microphone selection, model manager, AI cleanup, OpenCode Zen calls, or frontend frameworks.
