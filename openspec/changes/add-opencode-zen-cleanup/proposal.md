## Why

The current implementation completes the offline local Windows dictation flow but intentionally skips the `Cleaning` state and inserts the raw Whisper transcript. This change adds the mainspec-defined OpenCode Zen cleanup enhancement while preserving the core invariant that a successful Whisper transcript is never lost because Zen fails.

## What Changes

- Add a Rust-side OpenCode Zen cleanup service using the fixed OpenCode Zen provider and `https://opencode.ai/zen/v1/chat/completions` OpenAI-compatible Chat Completions endpoint.
- Route successful dictations through `Idle -> Recording -> Transcribing -> Cleaning -> Inserting -> Idle` instead of skipping `Cleaning`.
- Use `deepseek-v4-flash-free` as the default editable model ID while keeping provider and base URL non-editable.
- Add Settings UI fields for OpenCode Zen API Key and Model ID alongside the existing Start with Windows setting and fixed shortcut display.
- Store the API key outside plaintext settings JSON using Windows user credential storage in Windows release builds, with Linux development limited to an environment variable or gitignored local development config.
- Send only the current raw transcript to OpenCode Zen as a system message plus user message; never send audio, history, telemetry, clipboard content, foreground app context, or other application context.
- Keep the dictation cleanup prompt in one central Rust-side location and make the LLM behave only as a dictation cleanup engine, not an assistant.
- Apply a 10 second cleanup request timeout with no retry queue or resilience framework.
- Fall back to the raw Whisper transcript for all recoverable cleanup failures, including network errors, timeout, HTTP errors, unauthorized/invalid API key, rate limits, unavailable model, malformed responses, parse failures, and empty assistant content.
- Preserve existing tray-first lifecycle, fixed hotkey, local microphone capture, local Whisper transcription, offline fallback, active text input insertion, clipboard handling, autostart, single-dictation invariant, no audio persistence, and no transcript history.
- Do not add OpenCode Go, other providers, multiple provider architecture, prompt editor, assistant/chat mode, voice commands, history, telemetry, backend, database, retry queue, streaming requirement, realtime transcription, or translation mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-windows-dictation`: Change the post-Whisper flow to use OpenCode Zen cleanup before text insertion when available, add Zen settings and secure API key handling, and require raw transcript fallback whenever cleanup fails.

## Impact

- Affects `src-tauri/src/app_state.rs` by entering `Cleaning`, invoking cleanup after non-empty Whisper transcript, and passing cleaned-or-raw final text to insertion.
- Adds a small `src-tauri/src/services/cleanup.rs` Rust service for prompt ownership, request/response construction, timeout, validation, privacy boundaries, and fallback-oriented error reporting.
- Extends `src-tauri/src/services/settings.rs` for non-sensitive `model_id` persistence without writing API keys to settings JSON.
- Adds or enables secure credential storage dependency/code for Windows user credential storage; Linux development may use `OPENCODE_ZEN_API_KEY` or gitignored local development config only.
- Extends Tauri commands in `src-tauri/src/lib.rs` so Settings can save/read model ID and set/update API key without exposing the secret in normal settings responses or logs.
- Updates `src/index.html`, `src/main.ts`, and `src/styles.css` to show OpenCode Zen API Key, editable Model ID, Start with Windows, and fixed shortcut information using Vanilla TypeScript/HTML/CSS.
- May adjust `src-tauri/Cargo.toml` for secure credential storage support if existing dependencies are insufficient; `reqwest` is already present and remains the Rust HTTP client.
- Adds tests or targeted verification for cleanup payload construction, fallback behavior, settings persistence boundaries, and prompt/identifier preservation constraints where practical.
