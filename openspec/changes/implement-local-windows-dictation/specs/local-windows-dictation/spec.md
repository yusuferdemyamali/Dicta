## ADDED Requirements

### Requirement: Tray-first application shell
The application SHALL start as a Tauri 2 tray-first Windows desktop application without showing a large main window during normal startup.

#### Scenario: Application starts in tray
- **WHEN** the user launches the application normally
- **THEN** the application runs from the system tray without opening a main dashboard window

#### Scenario: Tray menu opens settings
- **WHEN** the user selects Settings from the tray menu
- **THEN** the application opens the Settings window

#### Scenario: Tray menu exits application
- **WHEN** the user selects Exit from the tray menu
- **THEN** the application terminates cleanly

### Requirement: Minimal settings and startup behavior
The application SHALL provide a Vanilla TypeScript/HTML/CSS Settings UI that shows Start with Windows and the fixed shortcut `Ctrl + Alt + Space`, and SHALL support user-level autostart through `tauri-plugin-autostart` with Start with Windows enabled by default.

#### Scenario: Settings shows required controls
- **WHEN** the user opens Settings
- **THEN** the UI shows Start with Windows and shortcut information for `Ctrl + Alt + Space`

#### Scenario: Autostart defaults enabled
- **WHEN** the application runs with no existing user setting
- **THEN** Start with Windows is treated as enabled and user-level autostart is enabled where available

#### Scenario: Autostart failure is non-fatal
- **WHEN** autostart cannot be enabled or disabled
- **THEN** the application continues running and informs the user without requiring Administrator privileges

### Requirement: Fixed global hotkey toggles recording
The application SHALL register `Ctrl + Alt + Space` with `tauri-plugin-global-shortcut` and use it as a fixed global toggle for dictation.

#### Scenario: Hotkey starts recording from idle
- **WHEN** the application state is `Idle` and the user presses `Ctrl + Alt + Space`
- **THEN** recording starts using the Windows default input device

#### Scenario: Hotkey stops recording
- **WHEN** the application state is `Recording` and the user presses `Ctrl + Alt + Space`
- **THEN** recording stops and transcription begins

#### Scenario: Hotkey works outside foreground application
- **WHEN** another application has foreground focus
- **THEN** pressing `Ctrl + Alt + Space` still triggers the dictation toggle

#### Scenario: Hotkey registration failure is non-fatal
- **WHEN** `Ctrl + Alt + Space` cannot be registered
- **THEN** the application remains running and informs the user that the shortcut is unavailable

### Requirement: Central dictation state machine
The application SHALL maintain one central Rust-owned dictation state with `Idle`, `Recording`, `Transcribing`, `Cleaning`, and `Inserting` states, and SHALL allow only one dictation operation at a time.

#### Scenario: Offline dictation state flow succeeds
- **WHEN** recording stops with captured speech audio
- **THEN** the state progresses through `Transcribing` and `Inserting` before returning to `Idle`

#### Scenario: Processing ignores new recording requests
- **WHEN** the state is `Transcribing`, `Cleaning`, or `Inserting` and the hotkey is pressed
- **THEN** no new recording starts and no dictation queue is created

#### Scenario: Cleaning state is present but not used for AI cleanup
- **WHEN** this offline dictation change is implemented
- **THEN** the state model includes `Cleaning` but no OpenCode Zen cleanup request is performed

### Requirement: Audio capture lifecycle
The application SHALL capture audio through `cpal` from the Windows default microphone only while in `Recording`, keep normal dictation audio in memory, and avoid writing temporary or persistent audio files during normal dictation.

#### Scenario: Microphone opens only during recording
- **WHEN** the application is `Idle`
- **THEN** microphone capture is not active

#### Scenario: Recording stores audio in memory
- **WHEN** the user starts and stops a normal dictation
- **THEN** captured audio is retained in memory for transcription and no audio history file is created

#### Scenario: Missing microphone is handled safely
- **WHEN** no default microphone is available or the microphone cannot be opened
- **THEN** the application does not crash, informs the user, and returns to `Idle`

### Requirement: Audio normalization for Whisper
The application SHALL normalize completed recordings to 16 kHz mono f32 PCM before sending audio to Whisper.

#### Scenario: Non-target audio format is converted
- **WHEN** the input device provides audio with a different sample rate, channel count, or sample format
- **THEN** the application converts the recording to 16 kHz mono f32 PCM before transcription

### Requirement: Local Whisper transcription
The application SHALL perform speech recognition locally using whisper.cpp through `whisper-rs`, with the default model set to Whisper Small Multilingual and language set to Turkish (`tr`).

#### Scenario: Speech is transcribed locally
- **WHEN** recording stops with valid audio
- **THEN** the application transcribes the audio locally without sending audio to a remote service

#### Scenario: Turkish transcription is configured
- **WHEN** Whisper inference runs
- **THEN** the transcription language is configured as Turkish (`tr`)

#### Scenario: Whisper returns raw transcript
- **WHEN** Whisper produces text
- **THEN** the application treats it as raw transcript and does not perform aggressive grammar cleanup or rewrite in the Whisper layer

### Requirement: Whisper model lifecycle
The application SHALL store the default Whisper model under the application local data models directory, download it if missing, skip redownload when present, and keep the loaded model in memory across dictations.

#### Scenario: Model downloads when missing
- **WHEN** the application starts and the default model file is missing
- **THEN** the application downloads the default model to the local app data models directory

#### Scenario: Existing model is reused
- **WHEN** the default model file already exists
- **THEN** the application does not download it again during normal startup

#### Scenario: Model stays loaded
- **WHEN** multiple dictations occur in one application session
- **THEN** the Whisper model is not reloaded for every dictation

#### Scenario: Inference does not block Tauri event loop
- **WHEN** Whisper inference runs
- **THEN** it runs on a blocking worker/thread/task rather than blocking the Tauri main async/event loop

### Requirement: Empty transcription handling
The application SHALL insert no text and SHALL return to `Idle` when Whisper produces an empty transcript.

#### Scenario: Empty transcript stops processing
- **WHEN** Whisper returns an empty or whitespace-only transcript
- **THEN** no text is inserted and the application state returns to `Idle`

### Requirement: Windows active text insertion
The application SHALL insert the final raw Whisper transcript into the currently focused Windows text input using clipboard plus Windows SendInput `Ctrl + V`, without bringing the Settings window to the foreground.

#### Scenario: Transcript pastes into focused input
- **WHEN** the user focuses a standard Windows text input in another application and completes dictation
- **THEN** the transcript is pasted into that focused input

#### Scenario: Turkish characters are preserved
- **WHEN** the transcript contains Turkish Unicode characters
- **THEN** the inserted text preserves those characters correctly

#### Scenario: Application-specific integrations are not required
- **WHEN** the focused target is a standard text input in Chrome, Firefox, Edge, VS Code, Notepad, Word, Telegram Desktop, Discord, Slack, or WhatsApp Desktop
- **THEN** insertion uses the general Windows clipboard and input mechanism rather than a per-application integration

### Requirement: Clipboard preservation
The application SHALL preserve the user's existing clipboard content on a best-effort basis when inserting dictation text.

#### Scenario: Clipboard is restored after paste
- **WHEN** text insertion completes after saving existing clipboard content
- **THEN** the previous clipboard content is restored where practical

#### Scenario: Clipboard failure is non-fatal
- **WHEN** clipboard save, paste, or restore fails
- **THEN** the application does not crash and returns to `Idle`

### Requirement: Recording feedback
The application SHALL provide minimal user feedback for recording and processing through tray state changes and short recording start/stop audio cues.

#### Scenario: Recording start feedback is shown
- **WHEN** recording starts
- **THEN** the tray state indicates recording and a short start cue is played

#### Scenario: Recording stop feedback is shown
- **WHEN** recording stops
- **THEN** a short stop cue is played and the tray state indicates processing while transcription or insertion is active

#### Scenario: Processing completes feedback
- **WHEN** insertion finishes or the flow ends safely after an error
- **THEN** the tray state returns to idle

### Requirement: Offline and privacy behavior
The application SHALL keep the core dictation flow offline by processing audio locally and SHALL not store audio history, transcript history, telemetry, or send audio to any external service.

#### Scenario: Internet is unavailable
- **WHEN** the user completes a dictation without internet connectivity
- **THEN** local Whisper transcription and text insertion still run

#### Scenario: Audio remains local
- **WHEN** transcription runs
- **THEN** audio is processed locally and is not sent to OpenCode Zen or any remote service

#### Scenario: History is not retained
- **WHEN** dictation completes
- **THEN** no audio history or transcript history is stored

### Requirement: Scope exclusions are enforced
The implementation SHALL NOT include OpenCode Zen cleanup calls, OpenCode Go, Python runtime, subprocess transcription services, local HTTP transcription services, databases, telemetry, transcript history, audio history, custom hotkeys, microphone selection UI, model manager UI, frontend frameworks, assistant/chat behavior, realtime streaming transcription, live subtitles, or floating rich UI.

#### Scenario: AI cleanup is absent
- **WHEN** the offline dictation flow completes
- **THEN** the inserted text is the raw Whisper transcript and no OpenCode Zen API request has been made

#### Scenario: No unsupported infrastructure is added
- **WHEN** the implementation is inspected
- **THEN** it contains no Python transcription runtime, no local transcription HTTP server, no database, and no frontend framework
