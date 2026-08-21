## Context

This repository currently contains the product source of truth (`mainspec.md`) and OpenSpec planning configuration, but no application implementation. The implementation should therefore create the initial Tauri 2 application structure while staying aligned with the simple service-based Rust layout from the mainspec.

The target product platform is Windows 10/11 x64. Linux is only a development environment, so Linux behavior may use small development fallbacks for Windows-only actions, but it must not become a supported product path or drive architecture choices.

This change implements the offline dictation slice only: fixed hotkey, recording, local Whisper transcription, and paste into the active Windows input. OpenCode Zen cleanup is intentionally deferred to a later change.

## Goals / Non-Goals

**Goals:**

- Create a tray-first Tauri 2 desktop application with Rust-owned core logic and Vanilla TypeScript/HTML/CSS Settings UI.
- Implement the fixed toggle hotkey `Ctrl + Alt + Space` using `tauri-plugin-global-shortcut`.
- Own dictation flow in one central application state machine: `Idle`, `Recording`, `Transcribing`, `Cleaning`, and `Inserting`.
- Capture audio only during `Recording`, keep normal audio in memory, and normalize to 16 kHz mono f32 PCM for Whisper.
- Use `whisper-rs`/whisper.cpp for fully local Turkish transcription with the Small Multilingual model.
- Keep the Whisper model loaded across multiple dictations and run blocking inference away from the Tauri event loop.
- Insert raw transcripts into the active Windows text field through clipboard plus SendInput `Ctrl + V`, preserving clipboard content on a best-effort basis.
- Provide minimal recording feedback through tray state and start/stop audio cues.
- Enable user-level autostart by default through `tauri-plugin-autostart`.

**Non-Goals:**

- No OpenCode Zen request, AI cleanup, prompt handling, OpenCode Go, or multiple AI providers.
- No custom hotkeys, push-to-talk mode, microphone selection, model manager, transcript/audio history, database, telemetry, assistant/chat mode, realtime transcription, live subtitles, or floating rich UI.
- No Python runtime, subprocess transcription service, local HTTP transcription service, frontend framework, or generic provider abstraction.
- No Linux product support or parity beyond development-safe fallbacks for Windows-only output.

## Decisions

### Central State Ownership

Use a single Rust `AppState` managed by Tauri state. It owns the current `DictationState`, the audio recorder handle, the transcription service, settings, startup state, and small UI feedback helpers. Hotkey callbacks delegate into a single `handle_hotkey()` path so state transitions remain auditable.

Alternatives considered:

- Frontend-owned state was rejected because global hotkey, audio capture, transcription, autostart, and text insertion are native concerns.
- Multiple per-service state machines were rejected because this MVP needs one serialized dictation flow and no queue.

The implemented state transitions are:

```text
Idle + hotkey -> Recording
Recording + hotkey -> Transcribing -> Inserting -> Idle
Transcribing/Cleaning/Inserting + hotkey -> ignored with non-crashing feedback/logging
```

`Cleaning` remains in the enum for mainspec alignment, but this change does not enter it for remote cleanup work.

### Simple Service Layout

Create the minimal Tauri structure:

```text
src/
├── index.html
├── main.ts
└── styles.css

src-tauri/src/
├── lib.rs
├── app_state.rs
└── services/
    ├── audio.rs
    ├── feedback.rs
    ├── hotkey.rs
    ├── settings.rs
    ├── startup.rs
    ├── text_output.rs
    └── transcription.rs
```

Windows-specific text insertion and any Windows-only clipboard or SendInput behavior should stay inside `services/text_output.rs` or a tiny child module guarded with `#[cfg(target_os = "windows")]`. Linux development fallback can write the transcript to stdout/log-safe development output or clipboard without claiming product support.

Alternatives considered:

- A plugin architecture, event bus, repository pattern, or job queue was rejected because it adds abstraction without solving the current single-flow problem.
- Splitting each platform into broad adapter trees was rejected; only text output needs explicit Windows boundaries at this stage.

### Audio Lifecycle

`AudioRecorder` uses `cpal` to open the default input device only when transitioning into `Recording`. Captured samples are appended to an in-memory buffer behind synchronization suitable for the cpal callback. On stop, the stream is dropped, the captured buffer is moved out, and conversion normalizes device sample format/channel count/sample rate to Whisper input: 16 kHz, mono, f32 PCM.

No normal dictation path writes WAV files or audio history. If the microphone is missing or cannot be opened, the transition fails safely back to `Idle` and the user is informed through tray/status feedback.

Alternatives considered:

- Temporary WAV files were rejected for privacy and lifecycle simplicity.
- Continuous microphone capture was rejected because idle resource use must remain low.
- A microphone picker was rejected as out of scope.

### Whisper Lifecycle

`TranscriptionService` resolves the model path under the app local data directory, such as `%LOCALAPPDATA%/<AppName>/models/`. On startup it ensures the default Small Multilingual model exists, downloading it if missing, then initializes a `whisper-rs` context once and keeps it in memory for reuse.

Inference runs in a blocking worker/thread/task, not directly on the Tauri event loop. A single in-flight dictation is allowed by the app state, so no transcription queue is needed. The Whisper language is fixed to Turkish (`tr`) for this change. Whisper returns raw transcript only; cleanup or rewrite behavior is not added.

Alternatives considered:

- Loading the model per dictation was rejected because it causes avoidable latency.
- A long-lived job queue was rejected because the state machine prevents concurrent dictations.
- Python, subprocesses, and local HTTP services were rejected by product constraints.

### Text Output And Clipboard

On Windows, `TextOutputService` saves the existing clipboard content when practical, writes the final transcript to the clipboard, sends `Ctrl + V` to the active focused application using Windows SendInput behavior, then restores the previous clipboard on a best-effort basis after paste has been initiated.

Clipboard + paste is chosen over character-by-character simulation because it is more reliable for long Unicode text and Turkish characters. The application must not foreground its Settings window during insertion.

Alternatives considered:

- Per-application integrations were rejected because the target is standard Windows text inputs across common applications.
- Character-by-character keyboard simulation was rejected due to Unicode reliability concerns.

### Settings And Startup

Settings UI is the only real window. It displays Start with Windows and the fixed shortcut. It may include inert OpenCode Zen API Key and Model ID fields only if useful to preserve the mainspec-compatible layout, but this change must not call Zen or require those values.

Start with Windows defaults to enabled through `tauri-plugin-autostart` at user level. If autostart cannot be enabled, the application continues and surfaces the failure without requiring Administrator privileges.

### Failure Behavior

Failures must not crash the app. The state returns to `Idle` after microphone, transcription, model, hotkey, or insertion failures. Empty transcription inserts nothing and stops processing. Hotkey registration failure leaves the app running in tray and informs the user that the shortcut is unavailable.

## Risks / Trade-offs

- Model download URL, size, or availability changes -> keep the download path/version explicit and skip redownload when the model exists.
- `whisper-rs` native build can differ between Linux dev and Windows release -> verify Windows build on a real Windows 10/11 x64 environment before accepting the change.
- Clipboard restore timing can race with target application paste handling -> keep restore best-effort and avoid complex synchronization that could break insertion reliability.
- Default device sample rates/channel layouts vary -> normalize all captured input before transcription and test with common 44.1 kHz/48 kHz mono/stereo inputs.
- Global hotkey can conflict with another application -> do not crash; expose the registration failure in Settings/tray feedback.

## Migration Plan

No persisted production data exists yet. Implementation can create the initial app files and default settings. Rollback is removing the new application files and dependencies for this change.

## Open Questions

None blocking. The exact Whisper Small Multilingual model filename and download source should be selected during implementation and documented in code/config without adding a model manager.
