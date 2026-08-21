<div align="center">

# 🎙️ Windows AI Dictation

**Speak naturally. Get clean, polished text anywhere on Windows.**

A lightweight, tray-based dictation app powered by local Whisper transcription and AI-assisted text cleanup through OpenCode Zen.

`Ctrl + Alt + Space` → Speak → `Ctrl + Alt + Space` → Done.

<br>

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square\&logo=tauri\&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-Core-000000?style=flat-square\&logo=rust\&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-Vanilla-3178C6?style=flat-square\&logo=typescript\&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D4?style=flat-square\&logo=windows11\&logoColor=white)
![Whisper](https://img.shields.io/badge/Whisper-Local-412991?style=flat-square)
![OpenCode Zen](https://img.shields.io/badge/OpenCode-Zen-111111?style=flat-square)

</div>

---

## ✨ What is it?

Windows AI Dictation is a small desktop application that turns natural speech into clean written text.

Instead of forcing you to speak like a transcription engine, you can talk normally:

> şey şimdi bizim sevkiyat sayfasında ürünleri raf adresine göre sıralayalım ama raf yoksa ürün adına göre alfabetik olsun yani raflılar önce gelsin

The app transcribes your speech locally, then cleans it up:

> Sevkiyat sayfasındaki ürünleri öncelikle raf adresine göre sıralayalım. Raf adresi olan ürünler önce gelsin. Raf adresi olmayan ürünler ise ürün adına göre alfabetik olarak sıralansın.

The final text is automatically inserted into the text field you were using.

---

## 🚀 How it works

```text
Ctrl + Alt + Space
        │
        ▼
   Start recording
        │
        ▼
      Speak
        │
        ▼
Ctrl + Alt + Space
        │
        ▼
    Stop recording
        │
        ▼
 Local Whisper
  transcription
        │
        ▼
   Raw transcript
        │
        ▼
  OpenCode Zen
   AI cleanup
        │
        ▼
   Cleaned text
        │
        ▼
 Paste into the
 active text field
```

The goal is simple:

**Hotkey → Speak → Hotkey → Text**

---

## 🧠 AI-assisted cleanup

This is not just speech-to-text.

Whisper handles transcription locally, while an LLM improves the resulting text.

The cleanup stage can:

* remove filler words such as `şey`, `eee`, `ııı`, and unnecessary repetitions,
* fix punctuation,
* improve sentence structure,
* convert spoken language into natural written language,
* correct obvious transcription mistakes when the intended meaning is clear,
* preserve technical terms, URLs, product codes and identifiers.

The AI is explicitly instructed **not to add information that was never spoken**.

It also does not behave like a chatbot.

If you dictate:

```text
bugün hava nasıl
```

the expected output is:

```text
Bugün hava nasıl?
```

—not an answer about the weather.

---

## 🔒 Local-first transcription

Audio is processed locally using:

**whisper.cpp + whisper-rs**

Your microphone recording is not sent to OpenCode Zen.

Only the resulting text transcript is sent for cleanup.

```text
Microphone
    │
    ▼
Local Whisper
    │
    ▼
Transcript
    │
    ▼
OpenCode Zen
```

Audio and transcript history are not stored by default.

---

## 🌐 Offline behavior

The core dictation functionality does not depend on an internet connection.

If OpenCode Zen cannot be reached:

```text
Audio
  ↓
Local Whisper
  ↓
Raw transcript
  ↓
Active text field
```

The raw Whisper transcript is inserted instead.

This means an API timeout, rate limit, unavailable model or internet outage should never cause a successful dictation to disappear.

---

## ⌨️ Global shortcut

The default shortcut is:

### `Ctrl + Alt + Space`

It works as a toggle.

| State      | Action                        |
| ---------- | ----------------------------- |
| Idle       | Start recording               |
| Recording  | Stop recording                |
| Processing | Ignore new recording requests |

You do not need to keep the keys pressed.

---

## 🖥️ Tray-first experience

The application is designed to stay out of the way.

When Windows starts, the app launches directly into the system tray.

The tray menu contains only the essentials:

```text
Settings
Exit
```

There is no dashboard or permanent application window.

The tray icon can indicate whether the app is:

* idle,
* recording,
* processing.

---

## ⚙️ Settings

The MVP settings window contains:

### OpenCode Zen API Key

Your personal OpenCode Zen API key.

### Model

Default:

```text
deepseek-v4-flash-free
```

The model ID is editable so another OpenCode Zen model can be used without rebuilding the application.

### Start with Windows

Controls whether the app starts automatically when you sign in to Windows.

---

## 🧩 Tech stack

| Area              | Technology                      |
| ----------------- | ------------------------------- |
| Desktop framework | Tauri 2                         |
| Core              | Rust                            |
| UI                | Vanilla TypeScript + HTML + CSS |
| Audio capture     | `cpal`                          |
| Speech-to-text    | `whisper.cpp` + `whisper-rs`    |
| HTTP client       | `reqwest`                       |
| AI provider       | OpenCode Zen                    |
| Global shortcut   | `tauri-plugin-global-shortcut`  |
| Autostart         | `tauri-plugin-autostart`        |
| Tray              | Tauri TrayIcon                  |
| Windows APIs      | `windows` crate                 |

No React.

No Electron.

No Python runtime.

No local web server.

No database.

---

## 🏗️ Architecture

The application intentionally uses a simple architecture.

```text
src/
├── index.html
├── main.ts
└── styles.css

src-tauri/
├── src/
│   ├── lib.rs
│   ├── app_state.rs
│   │
│   └── services/
│       ├── audio.rs
│       ├── transcription.rs
│       ├── cleanup.rs
│       ├── hotkey.rs
│       ├── text_output.rs
│       ├── settings.rs
│       └── startup.rs
│
├── Cargo.toml
└── tauri.conf.json
```

The main state flow is:

```text
Idle
 ↓
Recording
 ↓
Transcribing
 ↓
Cleaning
 ↓
Inserting
 ↓
Idle
```

Only one dictation can be processed at a time.

---

## 🔄 Processing flow

Conceptually, the application logic is intentionally small:

```text
Hotkey pressed

if Idle:
    start recording

if Recording:
    stop recording

    transcribe audio locally

    if transcription is empty:
        stop

    try:
        clean transcript with OpenCode Zen
    catch:
        use raw transcript

    insert final text into active application
```

The architecture should remain close to this level of complexity.

---

## 📝 Text insertion

After processing, the final text is inserted into the application you were already using.

The intended targets include:

* Chrome
* Firefox
* Edge
* VS Code
* Notepad
* Word
* Telegram Desktop
* Discord
* Slack
* WhatsApp Desktop
* other standard Windows text inputs

The preferred insertion strategy is:

```text
Final text
    ↓
Clipboard
    ↓
Ctrl + V
    ↓
Active field
```

This is more reliable for long Unicode text and Turkish characters than simulating every character individually.

Where possible, the previous clipboard content is restored afterwards.

---

## 🇹🇷 Turkish support

The initial product is primarily designed around Turkish dictation.

Whisper runs in multilingual mode with Turkish as the default language.

Examples of expected cleanup:

**Speech**

```text
eee mehmete şey yaz yarın saat on gibi gelicem
```

**Result**

```text
Mehmet'e yarın saat 10 gibi geleceğimi yaz.
```

Technical values should remain untouched whenever possible:

```text
00312453
SVK-260811-9B1A
Ctrl + Alt + Space
/api/orders/412
SQLSTATE[23502]
deepseek-v4-flash-free
```

---

## 🔐 Privacy & security

The application follows a local-first approach.

* Audio is processed locally.
* Audio is not uploaded to OpenCode Zen.
* Audio recordings are not permanently stored.
* Transcript history is not stored.
* Transcript contents are not written to normal application logs.
* API keys must never be committed to the repository.
* API keys must not be stored in plaintext production configuration.
* API keys must never appear in logs.

Only the current textual transcript is sent to OpenCode Zen for cleanup.

---

## 🛡️ Failure handling

The app should fail gracefully.

| Failure                     | Behavior                   |
| --------------------------- | -------------------------- |
| No microphone               | Recording is not started   |
| Microphone cannot be opened | Show error, keep app alive |
| Whisper returns empty text  | Insert nothing             |
| Zen timeout                 | Use raw transcript         |
| Zen HTTP error              | Use raw transcript         |
| Invalid API key             | Use raw transcript         |
| Rate limit                  | Use raw transcript         |
| Model unavailable           | Use raw transcript         |
| No internet                 | Use raw transcript         |
| Text insertion fails        | Do not crash               |

The user's successfully transcribed speech should not be lost just because AI cleanup failed.

---

## 🐧 Development

Development is primarily done on Linux.

The product itself targets:

```text
Windows 10 / 11 x64
```

Most application logic remains platform-independent:

```text
Audio capture
Transcription
AI cleanup
State management
Settings
```

Windows-specific behavior is kept in small isolated modules where necessary.

For example:

```rust
#[cfg(target_os = "windows")]
```

can be used for Windows-only integrations.

Windows releases should be built and tested in a real Windows environment, such as a Windows CI runner.

```text
Linux development
       ↓
      Git
       ↓
Windows CI runner
       ↓
 Tauri Windows build
       ↓
     Setup.exe
```

---

## 📦 MVP scope

The first version focuses exclusively on getting this workflow right:

```text
Windows starts
      ↓
App starts in tray
      ↓
Ctrl + Alt + Space
      ↓
Speak
      ↓
Ctrl + Alt + Space
      ↓
Local transcription
      ↓
AI cleanup
      ↓
Text appears
```

### Included

* global dictation shortcut,
* system tray,
* Windows autostart,
* local Whisper transcription,
* Turkish speech recognition,
* OpenCode Zen cleanup,
* automatic text insertion,
* offline fallback,
* minimal settings.

### Not included

The MVP intentionally does **not** include:

* account system,
* backend,
* database,
* transcript history,
* audio history,
* cloud sync,
* Linux release,
* macOS release,
* mobile app,
* multiple AI providers,
* OpenAI integration,
* Anthropic integration,
* Gemini integration,
* OpenCode Go,
* local LLM,
* custom prompts,
* prompt profiles,
* customizable hotkeys,
* microphone selector,
* model manager,
* translation mode,
* chat mode,
* voice commands,
* live transcription,
* floating overlay,
* analytics,
* telemetry,
* automatic updater.

These should not be added merely because they might be useful in the future.

---

## 🎯 Project philosophy

This project is **not a voice assistant**.

It is an:

> **AI-enhanced dictation tool.**

Whisper answers:

> “What did the user say?”

The LLM answers:

> “How should that same thought be written clearly?”

Nothing more.

---

## 🚧 Project status

**Early development / MVP**

Primary milestone:

> Reliably go from `Ctrl + Alt + Space` → speech → clean text → active Windows input with minimal latency.

---

<div align="center">

### Speak naturally. Write clearly.

**Local transcription · AI cleanup · No unnecessary complexity**

</div>
