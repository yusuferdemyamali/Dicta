use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static WHISPER_CTX: OnceLock<Mutex<Option<whisper_rs::WhisperContext>>> = OnceLock::new();

fn ctx_lock() -> &'static Mutex<Option<whisper_rs::WhisperContext>> {
    WHISPER_CTX.get_or_init(|| Mutex::new(None))
}

pub fn model_path(models_dir: &Path) -> PathBuf {
    models_dir.join("ggml-small.bin")
}

// Alternative small multilingual filename historically: ggml-small.bin or whisper-small.bin
// We keep ggml-small.bin as default and download URL documented.
const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin";

pub fn ensure_model_and_init(models_dir: &Path) -> anyhow::Result<()> {
    let path = model_path(models_dir);
    if !path.exists() {
        println!("Model missing, downloading from {}", MODEL_URL);
        download_model_blocking(MODEL_URL, &path)?;
    } else {
        println!("Model exists at {:?}", path);
    }

    init_whisper(&path)?;
    Ok(())
}

fn download_model_blocking(url: &str, dest: &Path) -> anyhow::Result<()> {
    // Use reqwest blocking via tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let resp = reqwest::get(url).await.map_err(|e| anyhow::anyhow!(e))?;
        if !resp.status().is_success() {
            anyhow::bail!("Model download failed: HTTP {}", resp.status());
        }
        let bytes = resp.bytes().await.map_err(|e| anyhow::anyhow!(e))?;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Write atomically: write to temp then rename
        let tmp = dest.with_extension("tmp");
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, dest)?;
        println!("Model downloaded to {:?} ({} bytes)", dest, bytes.len());
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}

fn init_whisper(model_path: &Path) -> anyhow::Result<()> {
    let mut guard = ctx_lock().lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }

    let params = whisper_rs::WhisperContextParameters::default();
    let ctx = whisper_rs::WhisperContext::new_with_params(model_path, params)
        .map_err(|e| anyhow::anyhow!("whisper init failed: {}", e))?;

    *guard = Some(ctx);
    Ok(())
}

pub fn transcribe_blocking(audio_16k_mono: Vec<f32>) -> anyhow::Result<String> {
    // If audio empty, return empty
    if audio_16k_mono.is_empty() {
        return Ok(String::new());
    }

    let guard = ctx_lock().lock().unwrap();
    let ctx = guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Whisper context not initialized"))?;

    // Create state
    let mut state = ctx
        .create_state()
        .map_err(|e| anyhow::anyhow!("create whisper state failed: {}", e))?;

    let mut params =
        whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("tr"));
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    // single thread default; could set threads to 4
    params.set_n_threads(4);

    state
        .full(params, &audio_16k_mono)
        .map_err(|e| anyhow::anyhow!("whisper inference failed: {}", e))?;

    let num_segments = state.full_n_segments();

    let mut transcript = String::new();
    for i in 0..num_segments {
        if let Some(seg) = state.get_segment(i) {
            if let Ok(text) = seg.to_str_lossy() {
                if !transcript.is_empty() {
                    transcript.push(' ');
                }
                transcript.push_str(text.trim());
            }
        }
    }

    // Return raw transcript without cleanup/rewrite
    Ok(transcript.trim().to_string())
}
