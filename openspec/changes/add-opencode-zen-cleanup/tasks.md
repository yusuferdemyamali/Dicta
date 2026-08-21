## 1. Dependencies And Module Structure

- [x] 1.1 Confirm existing `reqwest` and `tokio` dependencies satisfy the cleanup HTTP path; add only the smallest secure credential storage dependency if needed.
- [x] 1.2 Add `src-tauri/src/services/cleanup.rs` and export it from `src-tauri/src/services/mod.rs`.
- [x] 1.3 Add a small Rust credential helper module or functions for OpenCode Zen API key storage and retrieval.
- [x] 1.4 Keep Windows credential storage behind the chosen minimal implementation and keep Linux API key loading development-only via `OPENCODE_ZEN_API_KEY` or gitignored local config.

## 2. Settings And Secret Handling

- [x] 2.1 Extend `Settings` with `model_id` defaulting to `deepseek-v4-flash-free` while preserving `start_with_windows` default behavior.
- [x] 2.2 Ensure settings JSON persists only non-sensitive fields and never serializes the API key.
- [x] 2.3 Add Tauri commands to save/update the API key, report whether a key exists, and save/read model ID without returning the plaintext key.
- [x] 2.4 Update `save_settings` so autostart behavior remains unchanged and model ID persistence works with existing settings files.
- [x] 2.5 Ensure API key, raw transcript, cleaned transcript, and audio are never written to normal logs or error messages.

## 3. Settings UI

- [x] 3.1 Update `src/index.html` to include OpenCode Zen API Key, editable Model ID, Start with Windows, and fixed `Ctrl + Alt + Space` shortcut display.
- [x] 3.2 Update `src/main.ts` settings types and load/save behavior for model ID and API key status.
- [x] 3.3 Ensure the API key input does not preload the stored secret into the frontend.
- [x] 3.4 Keep provider and base URL out of editable UI controls.
- [x] 3.5 Adjust `src/styles.css` only as needed to keep the minimal Settings UI readable on desktop and narrow window sizes.

## 4. Cleanup Service

- [x] 4.1 Define fixed OpenCode Zen endpoint `https://opencode.ai/zen/v1/chat/completions`, default model, and 10 second timeout in the Rust cleanup service.
- [x] 4.2 Add the central dictation cleanup system prompt with rules for preserving meaning, original language, technical identifiers, questions, commands, URLs, product codes, stock codes, numbers, shortcuts, and model IDs.
- [x] 4.3 Implement OpenAI-compatible Chat Completions request structs with exactly one system message and one user message containing only the current raw transcript.
- [x] 4.4 Implement `reqwest` cleanup request execution with bearer API key authorization and no frontend request path.
- [x] 4.5 Parse the response and accept only non-empty assistant content as cleaned text.
- [x] 4.6 Convert timeout, network, DNS, HTTP 4xx/5xx, unauthorized, rate limit, model unavailable, malformed response, parse failure, and empty content into recoverable cleanup errors.
- [x] 4.7 Avoid retries, retry queues, streaming requirements, transcript history, conversation memory, telemetry, or extra app context in cleanup.

## 5. Pipeline Integration

- [x] 5.1 Update `AppState::process_recording` so non-empty Whisper transcripts enter `DictationState::Cleaning` before insertion.
- [x] 5.2 Call the cleanup service with the raw transcript, current model ID, and securely loaded API key.
- [x] 5.3 Use cleaned text for `Inserting` when cleanup succeeds.
- [x] 5.4 Use the raw Whisper transcript for `Inserting` when cleanup fails, times out, has no API key, or returns invalid content.
- [x] 5.5 Preserve the existing behavior for Whisper errors, empty transcripts, text insertion errors, tray feedback, and return to `Idle`.
- [x] 5.6 Confirm hotkey presses during `Cleaning` are ignored and no dictation queue is introduced.

## 6. Fallback, Privacy, And Logging Verification

- [x] 6.1 Add unit tests or targeted test helpers for cleanup request payload construction to verify only the current raw transcript is sent.
- [x] 6.2 Add tests or targeted verification for response parsing, empty assistant content, malformed JSON, and HTTP failure fallback.
- [x] 6.3 Add tests or targeted verification that settings JSON excludes the API key and includes model ID.
- [x] 6.4 Verify logs include only technical cleanup events such as completion, timeout, failure, and raw fallback without transcript text or secrets.
- [x] 6.5 Verify representative technical content preservation cases are covered by the central prompt or prompt tests: `00312453`, `SVK-260811-9B1A`, `Ctrl + Alt + Space`, `/api/orders/412`, `deepseek-v4-flash-free`, and `SQLSTATE[23502]`.

## 7. End-to-End Verification

- [x] 7.1 Run Rust formatting and check commands available in the project.
- [x] 7.2 Run frontend TypeScript/build checks available in the project.
- [x] 7.3 On Linux development, verify missing API key or network failure still produces raw transcript fallback without treating Linux as production support.
- [x] 7.4 On Windows 10/11 x64, verify Settings can save API key to secure credential storage and save editable model ID without plaintext key in settings JSON.
- [x] 7.5 On Windows 10/11 x64, verify `Ctrl + Alt + Space -> recording -> local Whisper -> Cleaning -> cleaned transcript -> active text input` works with a valid Zen key.
- [x] 7.6 On Windows 10/11 x64, verify invalid key, timeout, no internet, HTTP failure, malformed response, and empty cleanup content all insert the raw Whisper transcript.
- [x] 7.7 Verify existing tray-first lifecycle, fixed hotkey, local microphone capture, local Whisper transcription, offline fallback, active text input paste, clipboard handling, autostart, no audio persistence, and no transcript history remain intact.
