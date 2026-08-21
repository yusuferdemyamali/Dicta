## Context

The current repository has the completed `implement-local-windows-dictation` change archived and the corresponding `local-windows-dictation` spec synced into `openspec/specs/`. The app already has a Tauri 2 tray-first shell, fixed `Ctrl + Alt + Space` hotkey, local `cpal` recording, local `whisper-rs` transcription, clipboard-based active text insertion, autostart, and a central Rust `DictationState` containing `Cleaning`.

The current implementation intentionally skips remote cleanup: after a non-empty Whisper transcript, `src-tauri/src/app_state.rs` goes directly from `Transcribing` to `Inserting` and inserts the raw transcript. This change fills that deferred `Cleaning` step with OpenCode Zen while preserving the offline dictation behavior whenever cleanup is unavailable or fails.

Important current implementation details:

- `src-tauri/src/app_state.rs` owns the single-dictation state machine and processing thread.
- `src-tauri/src/services/settings.rs` persists only `start_with_windows` in local JSON today.
- `src/index.html` and `src/main.ts` currently expose Start with Windows, shortcut status, and app status only.
- `src-tauri/Cargo.toml` already includes `reqwest` and `tokio`; no frontend HTTP client is needed.
- `.gitignore` already excludes `.env`, `.env.local`, and local env variants.

## Goals / Non-Goals

**Goals:**

- Add OpenCode Zen cleanup after successful non-empty Whisper transcription.
- Preserve the state flow `Idle -> Recording -> Transcribing -> Cleaning -> Inserting -> Idle` for successful non-empty dictations.
- Guarantee that every recoverable cleanup failure inserts the raw Whisper transcript instead of dropping the dictation.
- Use Rust-side `reqwest` against `https://opencode.ai/zen/v1/chat/completions` with OpenAI-compatible Chat Completions JSON.
- Keep provider and base URL fixed to OpenCode Zen while allowing the Model ID setting to be edited.
- Keep the cleanup system prompt in one central Rust-side location.
- Store the API key outside plaintext settings JSON and outside logs/source control.
- Keep privacy boundaries strict: send only the current raw transcript text, with no audio, history, telemetry, clipboard content, foreground app metadata, or other context.
- Keep cleanup async or off the UI/event loop and preserve the existing single-dictation invariant.

**Non-Goals:**

- No OpenCode Go.
- No OpenAI, Anthropic, Gemini, local LLM, multiple provider architecture, plugin system, or provider abstraction beyond the fixed Zen service.
- No prompt editor, custom prompt profiles, assistant/chat mode, web search, command execution, voice commands, memory, history, cloud sync, backend, database, telemetry, analytics, retry queue, streaming requirement, realtime transcription, or translation mode.
- No frontend API requests.
- No Linux production secret-storage parity; Linux remains development-only.

## Decisions

### Cleanup Service Placement

Add `src-tauri/src/services/cleanup.rs` and export it from `services/mod.rs`. The service owns:

- fixed provider constants,
- fixed base URL,
- default model ID,
- central system prompt,
- request and response structs,
- `reqwest` client usage,
- 10 second timeout,
- assistant content validation,
- cleanup error classification that never includes transcript or API key content.

`app_state.rs` remains the orchestration owner. After Whisper returns a non-empty transcript, it sets `DictationState::Cleaning`, calls cleanup with the raw transcript and current settings/API key, then sets `DictationState::Inserting` with either cleaned text or raw fallback.

Alternatives considered:

- Putting cleanup in `transcription.rs` was rejected because Whisper must remain raw speech-to-text only.
- Creating a generic provider layer was rejected because the provider is fixed and multiple providers are out of scope.
- Frontend cleanup calls were rejected because API requests and secrets must stay Rust-side.

### Runtime Model And Async Boundary

The existing processing flow already runs transcription and insertion from a spawned worker thread so the Tauri event loop is not blocked by Whisper. Cleanup can use a small Tokio runtime inside that worker thread, or a blocking wait over an async `reqwest` call, as long as the UI/event loop is not blocked and no queue is introduced.

The cleanup request timeout is exactly 10 seconds for MVP. There is no retry. Timeout returns a cleanup error to `app_state.rs`; `app_state.rs` logs a technical failure event and uses raw transcript fallback.

Alternatives considered:

- A background job queue was rejected because single-dictation processing is an invariant.
- Retry/backoff was rejected because raw fallback is the required resilience behavior.
- Streaming response handling was rejected because MVP only needs final cleaned text.

### Zen Request Contract

The request is OpenAI-compatible Chat Completions JSON:

```json
{
  "model": "deepseek-v4-flash-free",
  "messages": [
    { "role": "system", "content": "<central cleanup system prompt>" },
    { "role": "user", "content": "<current raw transcript>" }
  ]
}
```

The service sends `Authorization: Bearer <api key>` and `Content-Type: application/json`. The only user-derived payload field is the current raw transcript. It must not attach audio, transcript history, clipboard text, active application details, telemetry, or any previous dictation.

The response is accepted only when the first assistant message has non-empty cleaned content after trimming. Empty content, missing choices, missing message/content, invalid JSON, or unexpected parse failures are cleanup failures and trigger raw fallback.

Alternatives considered:

- Sending extra app context was rejected for privacy and because dictation cleanup must remain stateless.
- Accepting empty cleanup output was rejected because it could erase a successful Whisper transcript.

### Prompt Ownership And Behavior

The cleanup prompt lives as a constant or small function in `services/cleanup.rs`. It is not editable in Settings and has no profile system. It must explicitly constrain the LLM to cleanup only:

- preserve meaning,
- do not add information,
- remove filler words and unnecessary repetition,
- correct punctuation, grammar, and sentence structure,
- preserve original language,
- preserve technical identifiers, URLs, product codes, stock codes, numbers, commands, shortcuts, model IDs, endpoint paths, and error codes,
- do not answer questions,
- do not execute dictated commands,
- return only final cleaned text without markdown, quotes, or explanation.

The prompt should include question handling as a hard rule. For example, `bugün hava nasıl` becomes `Bugün hava nasıl?`, not an answer.

Alternatives considered:

- Prompt editor and profiles were rejected because they increase scope and can weaken deterministic privacy/safety guarantees.

### Secure API Key Storage

Non-sensitive settings remain in the local JSON config. Add `model_id` there with default `deepseek-v4-flash-free`. Do not store the API key in this JSON.

For Windows release behavior, use Windows user-account-bound credential storage. The simplest implementation path is a small credential helper service, for example `services/credentials.rs`, backed by the Rust `keyring` crate if it is compatible with the current Tauri/Rust stack, or a tiny Windows Credential Manager wrapper if the crate adds unacceptable complexity. The credential entry should use a stable internal service name such as `dikte.opencode_zen` and account `default`.

For Linux development only, read `OPENCODE_ZEN_API_KEY` and optionally a gitignored local development config. This must be documented in code as development-only and not treated as production support.

Settings commands should let the UI save/update the API key and report whether a key exists. `get_settings` must not return the stored secret. Logs must never include the key.

Alternatives considered:

- Plaintext config storage was rejected by the mainspec.
- A custom secret-management abstraction was rejected; one small credential helper is enough.
- Linux keyring parity was rejected because Linux is not a product target.

### Settings UI Contract

Update the existing Settings window without adding a frontend framework. Minimum fields:

- OpenCode Zen API Key input,
- editable Model ID input defaulting to `deepseek-v4-flash-free`,
- Start with Windows checkbox,
- optional fixed shortcut display for `Ctrl + Alt + Space`.

Provider and base URL are not editable. The API key field can be write-only or display a placeholder/status like `Saved` when a key exists; it must not preload the secret value into the frontend. Save should persist model/autostart settings and update the secure API key only when the user enters a non-empty key or explicitly clears it if clear behavior is implemented.

Alternatives considered:

- Provider/base URL fields were rejected because they imply multiple providers or unsupported endpoint customization.
- Separate Zen settings window was rejected because the app has one minimal Settings UI.

### Logging And Fallback

Logging may include technical events only:

- `Cleanup completed`,
- `Cleanup request failed: timeout`,
- `Raw transcript fallback used`.

Logs must not include raw transcript content, cleaned transcript content, audio, or API key. The fallback matrix is implemented as one rule: any recoverable cleanup error returns raw transcript for insertion.

Alternatives considered:

- Exposing detailed response bodies in logs was rejected because provider responses could contain transcript content.
- Treating invalid API key as blocking was rejected because Zen is an enhancement, not a dependency.

## Risks / Trade-offs

- Windows credential storage dependency may add platform-specific build complexity -> keep it isolated in one helper and verify on Windows 10/11 x64.
- Cleanup can add up to 10 seconds latency when network is slow -> hard timeout and no retry keeps worst-case bounded.
- LLM may over-edit technical identifiers despite prompt rules -> central prompt emphasizes preservation and tests should include representative identifiers.
- Users without API key get raw fallback only -> show key status in Settings and preserve offline flow without blocking dictation.
- Linux development fallback for API key is weaker than Windows credential storage -> keep it explicitly development-only and gitignored/env-based.

## Migration Plan

No persisted production API key exists yet. Existing settings JSON can deserialize with default `model_id` and current `start_with_windows`; no compatibility layer beyond serde defaults is needed.

Implementation sequence:

1. Add cleanup and credential helpers.
2. Extend settings data and Tauri commands.
3. Update Settings UI.
4. Wire `Cleaning` into `app_state.rs` with raw fallback.
5. Verify offline fallback and Windows secure storage.

Rollback is safe: remove the cleanup call and UI/API key storage additions, and the offline raw transcript flow remains the same as the archived change.

## Open Questions

None blocking. During implementation, choose the smallest Windows credential storage approach that compiles cleanly with the current Tauri 2/Rust dependency set.
