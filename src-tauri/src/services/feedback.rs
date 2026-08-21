use crate::app_state::DictationState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::Duration;

pub fn update_tray_for_state(app: &tauri::AppHandle, state: DictationState) {
    let tooltip = match state {
        DictationState::Idle => "Dikte - Idle",
        DictationState::Recording => "Dikte - Recording ●",
        DictationState::Transcribing => "Dikte - Processing (Transcribing)",
        DictationState::Cleaning => "Dikte - Processing (Cleaning)",
        DictationState::Inserting => "Dikte - Processing (Inserting)",
    };
    // Update tray tooltip best-effort via known id "main" (set in lib.rs)
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    } else if let Some(tray) = app.tray_by_id("tray") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
    // Also log for debugging
    println!("Tray state: {:?}", tooltip);
}

fn play_tone(freq: f32, duration_ms: u64, volume: f32) {
    // Best-effort audio cue via cpal output stream, non-blocking failures
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("no output device for cue");
                return;
            }
        };
        let config = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("output config error: {}", e);
                return;
            }
        };
        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();

        let total_samples = (sample_rate * duration_ms as f32 / 1000.0) as usize;
        let mut sample_clock = 0usize;

        let err_fn = |err| eprintln!("cue stream error: {}", err);

        let stream_result = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    for frame in data.chunks_mut(channels) {
                        if sample_clock < total_samples {
                            let t = sample_clock as f32 / sample_rate;
                            // Simple sine wave with quick fade out
                            let envelope = 1.0 - (sample_clock as f32 / total_samples as f32) * 0.5;
                            let sample =
                                (2.0 * std::f32::consts::PI * freq * t).sin() * volume * envelope;
                            for ch in frame.iter_mut() {
                                *ch = sample;
                            }
                            sample_clock += 1;
                        } else {
                            for ch in frame.iter_mut() {
                                *ch = 0.0;
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    for frame in data.chunks_mut(channels) {
                        if sample_clock < total_samples {
                            let t = sample_clock as f32 / sample_rate;
                            let envelope = 1.0 - (sample_clock as f32 / total_samples as f32) * 0.5;
                            let sample =
                                (2.0 * std::f32::consts::PI * freq * t).sin() * volume * envelope;
                            let v = (sample * i16::MAX as f32) as i16;
                            for ch in frame.iter_mut() {
                                *ch = v;
                            }
                            sample_clock += 1;
                        } else {
                            for ch in frame.iter_mut() {
                                *ch = 0;
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _| {
                    for frame in data.chunks_mut(channels) {
                        if sample_clock < total_samples {
                            let t = sample_clock as f32 / sample_rate;
                            let envelope = 1.0 - (sample_clock as f32 / total_samples as f32) * 0.5;
                            let sample =
                                (2.0 * std::f32::consts::PI * freq * t).sin() * volume * envelope;
                            let v = ((sample + 1.0) * 0.5 * u16::MAX as f32) as u16;
                            for ch in frame.iter_mut() {
                                *ch = v;
                            }
                            sample_clock += 1;
                        } else {
                            for ch in frame.iter_mut() {
                                *ch = u16::MAX / 2;
                            }
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => {
                eprintln!("unsupported output format for cue");
                return;
            }
        };

        match stream_result {
            Ok(stream) => {
                if let Err(e) = stream.play() {
                    eprintln!("cue play failed: {}", e);
                    return;
                }
                std::thread::sleep(Duration::from_millis(duration_ms + 50));
                // stream dropped here
            }
            Err(e) => eprintln!("cue stream build failed: {}", e),
        }
    });
}

pub fn play_start_cue() {
    // Short beep 880Hz 120ms
    play_tone(880.0, 120, 0.25);
}

pub fn play_stop_cue() {
    // Lower beep 440Hz 120ms
    play_tone(440.0, 120, 0.25);
}
