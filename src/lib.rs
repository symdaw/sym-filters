#![feature(core_intrinsics)]
#![allow(nonstandard_style)]
#![allow(internal_features)]

mod biquad;
mod comb;
mod driven;
mod scream;
mod utils;

pub use biquad::Biquad;
pub use comb::Comb;
pub use driven::Driven;
pub use scream::Scream;

pub trait Filter : Clone {
    fn process(&mut self, data: &mut [Vec<f32>]) {
        for (channel_i, channel) in data.iter_mut().enumerate() {
            for frame in channel.iter_mut() {
                self.process_sample(frame, channel_i);
            }
        }
    }
    fn process_sample(&mut self, sample: &mut f32, channel_i: usize);

    fn transfer_function(&self, f: f32, sample_rate: f32) -> num_complex::Complex64 {
        // This is general purpose and works for any arbitrary filter but often you'll be able to
        // find a transfer function and should instead do that.

        let mut filter = self.clone();

        const SAMPLES: i32 = 100;
        let mut sum = 0.;
        for i in 0..1000 {
            let t = i as f32 / sample_rate;
            let phase = t * f * std::f32::consts::TAU;
            let mut sample = phase.sin();
            filter.process_sample(&mut sample, 0);
            sum += sample * sample;
        }
        let rms = (sum / SAMPLES as f32).sqrt();

        num_complex::Complex { re: rms as f64, im: 0. }
    }
    fn bandwidth(&self, sample_rate: f32) -> f32;

    fn amplitude_response(&self, f: f32, sample_rate: f32) -> f32 {
        //    G(ω) = |H(e^jωT)|
        self.transfer_function(f, sample_rate).norm() as f32
    }
    fn phase_response(&self, f: f32, sample_rate: f32) -> f32 {
        //    ϕ(ω) = ∠ H(e^jωT)
        self.transfer_function(f, sample_rate).arg() as f32
    }
}
