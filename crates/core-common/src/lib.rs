/// Common traits and structures for the emulator system core.
pub trait SystemCore {
    fn step(&mut self);
    fn step_frame(&mut self);
    fn audio_ready_for_frame(&self) -> bool;
    fn frame_buffer(&self) -> &[u32];
    fn set_input(&mut self, input: &ControllerState);
}

/// Represents the state of the game controller, including button presses.
#[derive(Default)]
pub struct ControllerState {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub trait AudioSink {
    /// Pushes audio samples to the sink for playback.
    /// 
    /// Returns the number of samples successfully pushed to the sink.
    fn push_samples(&mut self, samples: &[f32]) -> usize;

    /// Returns the number of audio samples that can be pushed to the sink without overflowing.
    fn space_available(&self) -> usize;

    /// Returns the sample rate of the audio sink in Hz.
    fn sample_rate(&self) -> u32;

    /// Returns the number of audio channels (e.g. 1 for mono, 2 for stereo). Defaults to 2.
    fn channels(&self) -> usize {
        2
    }
}

/// A no-op audio sink that discards all audio samples.
pub struct NullAudioSink;

impl AudioSink for NullAudioSink {
    fn push_samples(&mut self, samples: &[f32]) -> usize {
        // Since this is a null audio sink, we simply discard the samples.
        // We return the length of the input slice to indicate all samples were "pushed".
        samples.len()
    }

    fn space_available(&self) -> usize {
        // Since this is a null audio sink, it can always accept more samples.
        usize::MAX
    }

    fn sample_rate(&self) -> u32 {
        48000
    }
}
