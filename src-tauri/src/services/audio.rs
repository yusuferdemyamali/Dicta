use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    // Keep stream alive while recording
    stream: Option<cpal::Stream>,
    // In-memory buffer protected for cpal callback thread
    buffer: Arc<Mutex<Vec<f32>>>,
    // Device sample rate and channels for conversion needed
    sample_rate: u32,
    channels: u16,
    sample_format: cpal::SampleFormat,
    // Captured raw interleaved float buffer? We store f32 after conversion from callback
    // Actually callback will push f32 normalized samples
}

// cpal Stream is !Send on some platforms but we need AppState: Send for Tauri managed state.
// Safety: Stream is kept alive only while recording and accessed only from the thread that created it via Mutex.
// For MVP Linux dev we allow Send/Sync via unsafe impls to satisfy Tauri's Send bound. Real Windows usage keeps stream on same thread lifecycle.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 48000,
            channels: 1,
            sample_format: cpal::SampleFormat::F32,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No default input device"))?;

        let supported_config = device
            .default_input_config()
            .map_err(|e| anyhow::anyhow!("Failed to get default input config: {}", e))?;

        self.sample_rate = supported_config.sample_rate().0;
        self.channels = supported_config.channels();
        self.sample_format = supported_config.sample_format();

        let config: cpal::StreamConfig = supported_config.into();
        let buffer_clone = self.buffer.clone();
        let _channels = self.channels as usize;

        let err_fn = |err| eprintln!("cpal stream error: {}", err);

        let stream = match self.sample_format {
            cpal::SampleFormat::F32 => {
                let b = buffer_clone.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _| {
                        let mut buf = b.lock().unwrap();
                        // data is interleaved with channels
                        buf.extend_from_slice(data);
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::I16 => {
                let b = buffer_clone.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _| {
                        let mut buf = b.lock().unwrap();
                        for &s in data {
                            buf.push(s as f32 / i16::MAX as f32);
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::U16 => {
                let b = buffer_clone.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[u16], _| {
                        let mut buf = b.lock().unwrap();
                        for &s in data {
                            // normalize u16 0..65535 to -1..1 via f32
                            let v = (s as f32 / 65535.0) * 2.0 - 1.0;
                            buf.push(v);
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            other => {
                return Err(anyhow::anyhow!("Unsupported sample format: {:?}", other));
            }
        };

        stream.play()?;
        self.stream = Some(stream);
        // Clear buffer at start
        self.buffer.lock().unwrap().clear();
        println!(
            "AudioRecorder started: {} Hz, {} ch, {:?}",
            self.sample_rate, self.channels, self.sample_format
        );
        Ok(())
    }

    pub fn stop(&mut self) {
        // Drop stream to stop capture
        self.stream = None;
        println!("AudioRecorder stopped");
    }

    pub fn take_buffer(&mut self) -> Vec<f32> {
        let data = self.buffer.lock().unwrap().clone();
        // Return interleaved f32 with original channels/sampleRate
        // Convert to 16 kHz mono f32 PCM
        self.convert_to_16k_mono(&data)
    }

    fn convert_to_16k_mono(&self, interleaved: &[f32]) -> Vec<f32> {
        if interleaved.is_empty() {
            return Vec::new();
        }
        // Step 1: mono conversion by averaging channels
        let channels = self.channels as usize;
        let frames = interleaved.len() / channels;
        let mut mono: Vec<f32> = Vec::with_capacity(frames);
        for frame_idx in 0..frames {
            let mut sum = 0.0;
            for ch in 0..channels {
                sum += interleaved[frame_idx * channels + ch];
            }
            mono.push(sum / channels as f32);
        }

        // Step 2: resample to 16kHz if needed (simple linear interpolation)
        let target_rate = 16000u32;
        if self.sample_rate == target_rate {
            return mono;
        }

        let ratio = self.sample_rate as f32 / target_rate as f32;
        let target_len = ((mono.len() as f32) / ratio).ceil() as usize;
        let mut resampled = Vec::with_capacity(target_len);
        for i in 0..target_len {
            let src_pos = i as f32 * ratio;
            let src_idx = src_pos.floor() as usize;
            let frac = src_pos - src_idx as f32;
            if src_idx + 1 < mono.len() {
                let a = mono[src_idx];
                let b = mono[src_idx + 1];
                resampled.push(a * (1.0 - frac) + b * frac);
            } else if src_idx < mono.len() {
                resampled.push(mono[src_idx]);
            }
        }
        resampled
    }
}
