const CPU_M_CYCLES_PER_SECOND: u32 = 1_048_576;
const CPU_T_CYCLES_PER_SECOND: f32 = CPU_M_CYCLES_PER_SECOND as f32 * 4.0;

/// Factor used in the high-pass filter to remove DC offset and low-frequency noise.
const DC_REMOVAL_FACTOR: f32 = 0.999958;

/// Audio output handler that manages the audio buffer, sample rate, and
/// high-pass filtering for stereo audio output.
pub struct AudioOutput {
    buffer: Vec<f32>, // Internal audio buffer storing accumulated stereo samples before draining to the output vector
    sample_rate: u32, // The number of audio samples per second, used to control the timing of audio processing and high-pass filtering
    sample_counter: u32, // Counter tracking the number of audio samples processed relative to CPU M-cycles
    hp_l: HighPassFilter, // High-pass filter for the left audio channel to remove DC offset and low-frequency noise
    hp_r: HighPassFilter, // High-pass filter for the right audio channel to remove DC offset and low-frequency noise
    charge: f32, // Charge factor used in the high-pass filter to control the responsiveness to changes in the input signal
    acc_l: f32, // Accumulator for the left audio channel, storing the sum of samples before averaging
    acc_r: f32, // Accumulator for the right audio channel, storing the sum of samples before averaging
    acc_n: u32, // Counter for the number of accumulated audio samples, used to calculate the average before filtering
}

impl AudioOutput {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(2048),
            sample_rate: 48000,
            sample_counter: 0,
            hp_l: HighPassFilter::new(),
            hp_r: HighPassFilter::new(),
            charge: 0.0,
            acc_l: 0.0,
            acc_r: 0.0,
            acc_n: 0,
        }
    }

    /// Drains the current audio buffer into the provided output vector.
    /// This clears the internal audio buffer after copying its contents.
    pub fn drain_audio(&mut self, out: &mut Vec<f32>) {
        out.clear();
        out.extend_from_slice(&self.buffer);
        self.buffer.clear();
    }

    /// Sets the audio sample rate for the output.
    /// The `sample_rate` parameter specifies the number of audio samples per second.
    /// This affects the timing of audio sample accumulation and high-pass filtering.
    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        if sample_rate > 0 {
            self.sample_rate = sample_rate;
            self.sample_counter = 0;
            self.charge = DC_REMOVAL_FACTOR.powf(CPU_T_CYCLES_PER_SECOND / sample_rate as f32);
        }
    }

    /// Processes a stereo audio sample from the APU, accumulating it for later averaging and high-pass filtering.
    /// The `stereo_sample` parameter is a tuple containing the left and right audio channel samples.
    pub fn sample_audio(&mut self, stereo_sample: (f32, f32)) {
        // Accumulate audio samples from the APU for later averaging and filtering
        let (l, r) = stereo_sample;
        self.acc_l += l;
        self.acc_r += r;
        self.acc_n += 1;

        self.sample_counter += self.sample_rate;
        while self.sample_counter >= CPU_M_CYCLES_PER_SECOND {
            // Calculate the average of the accumulated audio samples for the left and right channels
            // This helps to smooth out the audio and prepare it for high-pass filtering
            self.sample_counter -= CPU_M_CYCLES_PER_SECOND;
            let n = self.acc_n.max(1) as f32;
            let (l, r) = (self.acc_l / n, self.acc_r / n);
            self.acc_l = 0.0;
            self.acc_r = 0.0;
            self.acc_n = 0;

            // Apply high-pass filter to the left and right audio channels to remove DC offset
            // and low-frequency noise
            let l = self.hp_l.process(l, self.charge);
            let r = self.hp_r.process(r, self.charge);

            // Push the filtered audio samples to the audio buffer if there is space available
            if self.buffer.len() < 8192 {
                self.buffer.push(l);
                self.buffer.push(r);
            }
        }
    }
}

/// High-pass filter for audio processing
/// The `cap` field stores the capacitor charge, which is used to calculate the high-pass output.
struct HighPassFilter {
    cap: f32,
}

impl HighPassFilter {
    /// Creates a new HighPass filter with the initial capacitor charge set to 0.0.
    fn new() -> Self {
        Self { cap: 0.0 }
    }

    /// The `process` method applies the high-pass filter to the input sample using the specified charge factor.
    /// The `input` parameter is the current audio sample, and the `charge` parameter determines how quickly the filter responds to changes in the input signal.
    pub fn process(&mut self, input: f32, charge: f32) -> f32 {
        let out = input - self.cap;
        self.cap = input - out * charge;
        out
    }
}
