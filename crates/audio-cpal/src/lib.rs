use core_common::AudioSink;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};

pub struct CpalAudioSink {
    _stream: cpal::Stream,
    producer: ringbuf::HeapProd<f32>,
    sample_rate: u32,
    channels: usize,
}

impl CpalAudioSink {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No output device found")?;
        let config = device.default_output_config()?.config();
        let sample_rate = config.sample_rate;
        let channels = config.channels as usize;

        println!(
            "Initializing CpalAudioSink with sample rate: {} Hz, channels: {}",
            sample_rate, channels
        );

        let capacity = ((sample_rate as usize / 20) * channels) * 2;
        let rb = HeapRb::<f32>::new(capacity);
        let (producer, mut consumer) = rb.split();

        let mut last = (0.0f32, 0.0f32);
        let stream = device.build_output_stream(
            config,
            move |out: &mut [f32], _| {
                for frame in out.chunks_mut(channels) {
                    if channels == 1 {
                        // If we have at least 1 sample available,
                        // pop it and update `last.0` with the new value
                        if let Some(s) = consumer.try_pop() {
                            last.0 = s;
                        }
                        frame[0] = last.0;
                    } else if channels >= 2 {
                        // If we have at least 2 samples available,
                        // pop them and update `last` with the new values
                        if consumer.occupied_len() >= 2 {
                            last = (consumer.try_pop().unwrap(), consumer.try_pop().unwrap());
                        }
                        frame[0] = last.0;
                        frame[1] = last.1;
                        for extra in &mut frame[2..] {
                            *extra = 0.0;
                        }
                    }
                }
            },
            |err| eprintln!("CpalAudioSink stream error: {err}"),
            None,
        )?;

        stream.play()?;

        Ok(Self {
            _stream: stream,
            producer,
            sample_rate,
            channels,
        })
    }
}

impl AudioSink for CpalAudioSink {
    fn push_samples(&mut self, samples: &[f32]) -> usize {
        if self.channels == 1 {
            // For mono audio, we need to downmix stereo input to a single channel
            let mut pushed = 0;
            for chunk in samples.chunks(2) {
                let mono = if chunk.len() == 2 {
                    (chunk[0] + chunk[1]) * 0.5
                } else {
                    chunk[0]
                };
                if self.producer.try_push(mono).is_ok() {
                    pushed += chunk.len();
                } else {
                    break;
                }
            }
            pushed
        } else {
            // For stereo or multi-channel audio, push as many whole L/R pairs as possible
            let n = samples.len().min(self.space_available()) & !1;
            self.producer.push_slice(&samples[..n])
        }
    }

    fn space_available(&self) -> usize {
        self.producer.capacity().get() - self.producer.occupied_len()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> usize {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpal_audio_sink_instantiation() {
        match CpalAudioSink::new() {
            Ok(mut sink) => {
                assert!(sink.sample_rate() > 0);
                assert!(sink.channels() > 0);
                let samples = vec![0.0f32; 100];
                let pushed = sink.push_samples(&samples);
                assert!(pushed > 0);
            }
            Err(e) => {
                // In headless or containerized environments without an audio device,
                // returning an error is expected.
                eprintln!("No audio device available in test environment: {e}");
            }
        }
    }
}
