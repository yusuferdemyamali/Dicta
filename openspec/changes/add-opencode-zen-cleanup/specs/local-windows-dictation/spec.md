## ADDED Requirements

### Requirement: OpenCode Zen cleanup service
The application SHALL add a Rust-side OpenCode Zen cleanup service that submits successful non-empty Whisper transcripts to OpenCode Zen before text insertion.

#### Scenario: Cleanup service uses fixed Zen endpoint
- **WHEN** the application performs AI cleanup
- **THEN** it sends the request from Rust to `https://opencode.ai/zen/v1/chat/completions` using `reqwest`

#### Scenario: Cleanup service uses Chat Completions format
- **WHEN** the application builds the cleanup request
- **THEN** it uses OpenAI-compatible Chat Completions JSON with one system message and one user message containing the current raw transcript

#### Scenario: OpenCode Go is not used
- **WHEN** the cleanup implementation is inspected
- **THEN** it uses OpenCode Zen and contains no OpenCode Go integration

### Requirement: Cleanup model configuration
The application SHALL default the cleanup model ID to `deepseek-v4-flash-free`, SHALL allow the user to edit the model ID, and SHALL keep provider and base URL fixed.

#### Scenario: Default model is applied
- **WHEN** the user has not configured a model ID
- **THEN** cleanup requests use `deepseek-v4-flash-free`

#### Scenario: Model ID is editable
- **WHEN** the user saves a different model ID in Settings
- **THEN** subsequent cleanup requests use that saved model ID

#### Scenario: Provider and base URL are not editable
- **WHEN** the user opens Settings
- **THEN** the UI does not provide controls to change provider or base URL

### Requirement: Secure OpenCode Zen API key handling
The application SHALL store the OpenCode Zen API key outside source code, git-tracked files, plaintext settings JSON, and logs.

#### Scenario: API key is saved securely on Windows
- **WHEN** the user saves an OpenCode Zen API key in a Windows release environment
- **THEN** the key is stored in Windows user-account-bound secure credential storage

#### Scenario: API key is not returned to frontend settings reads
- **WHEN** the frontend requests current settings
- **THEN** the response does not include the plaintext API key

#### Scenario: API key is absent from settings JSON
- **WHEN** the local settings JSON is inspected after saving settings
- **THEN** it contains non-sensitive settings such as model ID and Start with Windows but not the API key

#### Scenario: Linux API key source is development-only
- **WHEN** the application runs in the Linux development environment
- **THEN** it may read the API key from `OPENCODE_ZEN_API_KEY` or a gitignored local development config without treating that behavior as production support

### Requirement: Cleanup prompt behavior
The application SHALL keep the dictation cleanup system prompt in one central Rust-side location and SHALL instruct the LLM to return only cleaned dictation text.

#### Scenario: Prompt preserves meaning and language
- **WHEN** the cleanup prompt is used
- **THEN** it instructs the LLM to preserve the user's meaning and original language while improving written clarity

#### Scenario: Prompt forbids added information
- **WHEN** the cleanup prompt is used
- **THEN** it instructs the LLM not to add facts, requirements, assumptions, examples, or ideas the user did not say

#### Scenario: Prompt forbids assistant behavior
- **WHEN** the cleanup prompt is used
- **THEN** it instructs the LLM not to answer questions, execute dictated commands, explain changes, wrap output in markdown, or quote the output

#### Scenario: Question dictation remains a question
- **WHEN** the raw transcript is `bugün hava nasıl`
- **THEN** acceptable cleanup is semantically `Bugün hava nasıl?` and not an answer such as `Bugün hava güneşli.`

### Requirement: Technical content preservation
The cleanup prompt and validation strategy SHALL preserve technical identifiers and SHALL NOT aggressively rewrite ambiguous technical content.

#### Scenario: Identifier examples are preserved
- **WHEN** the raw transcript contains values such as `00312453`, `SVK-260811-9B1A`, `Ctrl + Alt + Space`, `/api/orders/412`, `deepseek-v4-flash-free`, or `SQLSTATE[23502]`
- **THEN** cleanup preserves those values unless the surrounding dictation makes a correction unambiguous

#### Scenario: Model identifier is not guessed
- **WHEN** a model identifier appears in the transcript and the intended correction is unclear
- **THEN** cleanup leaves the model identifier unchanged

### Requirement: Cleanup privacy boundary
The application SHALL send only the current raw transcript text to OpenCode Zen and SHALL NOT send audio or unrelated application context.

#### Scenario: Only current transcript is sent
- **WHEN** a cleanup request is made
- **THEN** the only user dictation content sent is the current raw Whisper transcript

#### Scenario: Audio is never sent to Zen
- **WHEN** a cleanup request is made
- **THEN** no audio data is included in the request

#### Scenario: No history or app context is sent
- **WHEN** a cleanup request is made
- **THEN** no conversation history, previous dictations, transcript history, telemetry, clipboard content, foreground application information, or other application context is included

### Requirement: Cleanup timeout and fallback
The application SHALL apply a 10 second timeout to each OpenCode Zen cleanup request and SHALL use the raw Whisper transcript whenever cleanup does not produce valid cleaned text.

#### Scenario: Cleanup request times out
- **WHEN** OpenCode Zen does not return valid cleaned text within 10 seconds
- **THEN** the application inserts the raw Whisper transcript

#### Scenario: Network or DNS failure falls back
- **WHEN** the cleanup request fails because internet is unavailable, DNS fails, or a network request error occurs
- **THEN** the application inserts the raw Whisper transcript

#### Scenario: HTTP failure falls back
- **WHEN** OpenCode Zen returns HTTP 4xx, HTTP 5xx, unauthorized, rate limit, invalid API key, or model unavailable responses
- **THEN** the application inserts the raw Whisper transcript

#### Scenario: Malformed response falls back
- **WHEN** the cleanup response cannot be parsed, has an unexpected shape, or contains empty assistant content
- **THEN** the application inserts the raw Whisper transcript

#### Scenario: No retry queue is created
- **WHEN** a cleanup request fails
- **THEN** the application does not queue the transcript for retry and continues with raw fallback

### Requirement: Cleanup-safe logging
The application SHALL log cleanup technical events without logging secrets, transcript content, cleaned content, or audio.

#### Scenario: Technical cleanup event is logged
- **WHEN** cleanup succeeds, fails, times out, or raw fallback is used
- **THEN** the application may log a technical event such as `Cleanup completed`, `Cleanup request failed: timeout`, or `Raw transcript fallback used`

#### Scenario: Sensitive cleanup data is not logged
- **WHEN** cleanup succeeds or fails
- **THEN** logs do not include the API key, raw transcript content, cleaned transcript content, or audio

## MODIFIED Requirements

### Requirement: Minimal settings and startup behavior
The application SHALL provide a Vanilla TypeScript/HTML/CSS Settings UI that shows OpenCode Zen API Key, editable Model ID, Start with Windows, and the fixed shortcut `Ctrl + Alt + Space`, and SHALL support user-level autostart through `tauri-plugin-autostart` with Start with Windows enabled by default.

#### Scenario: Settings shows required controls
- **WHEN** the user opens Settings
- **THEN** the UI shows OpenCode Zen API Key, editable Model ID, Start with Windows, and shortcut information for `Ctrl + Alt + Space`

#### Scenario: Autostart defaults enabled
- **WHEN** the application runs with no existing user setting
- **THEN** Start with Windows is treated as enabled and user-level autostart is enabled where available

#### Scenario: Autostart failure is non-fatal
- **WHEN** autostart cannot be enabled or disabled
- **THEN** the application continues running and informs the user without requiring Administrator privileges

### Requirement: Central dictation state machine
The application SHALL maintain one central Rust-owned dictation state with `Idle`, `Recording`, `Transcribing`, `Cleaning`, and `Inserting` states, SHALL allow only one dictation operation at a time, and SHALL route successful non-empty Whisper transcripts through `Cleaning` before `Inserting`.

#### Scenario: Zen-enhanced dictation state flow succeeds
- **WHEN** recording stops, Whisper produces a non-empty raw transcript, and OpenCode Zen cleanup succeeds
- **THEN** the state progresses through `Transcribing`, `Cleaning`, and `Inserting` before returning to `Idle`

#### Scenario: Processing ignores new recording requests
- **WHEN** the state is `Transcribing`, `Cleaning`, or `Inserting` and the hotkey is pressed
- **THEN** no new recording starts and no dictation queue is created

#### Scenario: Cleaning state performs AI cleanup
- **WHEN** Whisper produces a non-empty raw transcript
- **THEN** the application enters `Cleaning` and attempts OpenCode Zen cleanup before insertion

#### Scenario: Cleaning failure does not lose dictation
- **WHEN** Whisper produces a non-empty raw transcript and cleanup fails
- **THEN** the application proceeds to `Inserting` with the raw Whisper transcript and then returns to `Idle`

### Requirement: Windows active text insertion
The application SHALL insert the final dictation text into the currently focused Windows text input using clipboard plus Windows SendInput `Ctrl + V`, without bringing the Settings window to the foreground. The final dictation text SHALL be the cleaned transcript when cleanup succeeds and the raw Whisper transcript when cleanup fails or is unavailable.

#### Scenario: Cleaned transcript pastes into focused input
- **WHEN** the user focuses a standard Windows text input in another application and completes dictation with successful cleanup
- **THEN** the cleaned transcript is pasted into that focused input

#### Scenario: Raw transcript pastes after cleanup failure
- **WHEN** the user focuses a standard Windows text input in another application and cleanup fails after successful Whisper transcription
- **THEN** the raw Whisper transcript is pasted into that focused input

#### Scenario: Turkish characters are preserved
- **WHEN** the final dictation text contains Turkish Unicode characters
- **THEN** the inserted text preserves those characters correctly

#### Scenario: Application-specific integrations are not required
- **WHEN** the focused target is a standard text input in Chrome, Firefox, Edge, VS Code, Notepad, Word, Telegram Desktop, Discord, Slack, or WhatsApp Desktop
- **THEN** insertion uses the general Windows clipboard and input mechanism rather than a per-application integration

### Requirement: Offline and privacy behavior
The application SHALL keep local Whisper transcription offline by processing audio locally, SHALL use OpenCode Zen only as an optional text cleanup enhancement, SHALL send only the current raw transcript text to Zen, and SHALL not store audio history, transcript history, or telemetry.

#### Scenario: Internet is unavailable
- **WHEN** the user completes a dictation without internet connectivity
- **THEN** local Whisper transcription and text insertion still run using the raw transcript fallback

#### Scenario: Audio remains local
- **WHEN** transcription and cleanup run
- **THEN** audio is processed locally and is not sent to OpenCode Zen or any remote service

#### Scenario: Only transcript is sent for cleanup
- **WHEN** cleanup runs successfully
- **THEN** OpenCode Zen receives only the current textual raw transcript and no audio or history

#### Scenario: History is not retained
- **WHEN** dictation completes
- **THEN** no audio history or transcript history is stored

### Requirement: Scope exclusions are enforced
The implementation SHALL NOT include OpenCode Go, OpenAI provider, Anthropic provider, Gemini provider, multiple AI provider architecture, provider plugin systems, local LLM, prompt editor, custom prompt profiles, Python runtime, subprocess transcription services, local HTTP transcription services, databases, telemetry, transcript history, audio history, custom hotkeys, microphone selection UI, model manager UI, frontend frameworks, assistant/chat behavior, web search, command execution, voice commands, realtime streaming transcription, live subtitles, translation mode, cloud sync, backend, analytics, retry queue, or floating rich UI.

#### Scenario: OpenCode Zen cleanup is the only cloud enhancement
- **WHEN** the Zen cleanup flow completes or fails
- **THEN** the application has used only the fixed OpenCode Zen cleanup provider and no other AI provider or assistant mode

#### Scenario: No unsupported infrastructure is added
- **WHEN** the implementation is inspected
- **THEN** it contains no Python transcription runtime, no local transcription HTTP server, no database, no telemetry, no history system, and no frontend framework

#### Scenario: No command execution behavior is added
- **WHEN** dictated text contains an instruction or command
- **THEN** the application treats it as dictation text for cleanup and insertion, not as an action to execute
