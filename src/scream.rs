use crate::{Biquad, Filter};

#[derive(Default)]
pub struct Scream {
    hpf: Biquad,
    lpf: Biquad,
    prev_samples: [f32; 16],
    feedback_gain: f32,
    input_volume: [f32; 16],
}

impl Filter for Scream {
    fn process(&mut self, data: &mut [Vec<f32>]) {
        for (channel_i, channel) in data.iter_mut().enumerate() {
            for sample_ref in channel.iter_mut() {
                self.input_volume[channel_i] =
                    lerp(self.input_volume[channel_i], sample_ref.abs(), 0.01);

                let mut feedback_gain = self.feedback_gain;

                const THRESHOLD: f32 = 0.05;
                if self.input_volume[channel_i] < THRESHOLD {
                    feedback_gain *= self.input_volume[channel_i] / THRESHOLD;
                }

                // Based on https://github.com/Speechrezz/Scream-Filter
                let mut sample = (*sample_ref + self.prev_samples[channel_i]).tanh();
                self.lpf.process_sample(&mut sample, channel_i);
                *sample_ref = sample;
                self.hpf.process_sample(&mut sample, channel_i);
                sample = (sample * feedback_gain).tanh();
                self.prev_samples[channel_i] = sample;
            }
        }
    }

    fn transfer_function(&self, f: f32, sample_rate: f32) -> num_complex::Complex64 {
        self.hpf.transfer_function(f, sample_rate) * self.lpf.transfer_function(f, sample_rate)
    }

    fn bandwidth(&self, _sample_rate: f32) -> f32 {
        todo!()
    }
}

impl Scream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_params(&mut self, scream: f32, f: f32, resonance: f32, sample_rate: f32) {
        self.lpf.lpf(f + scream, 1., sample_rate);
        self.hpf.hpf(scream, 1., sample_rate);
        self.feedback_gain = resonance;
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
