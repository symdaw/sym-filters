use std::intrinsics::{fadd_fast, fmul_fast, fsub_fast};

use num_complex::Complex64;

use crate::Filter;

#[derive(Clone)]
pub struct Comb<const Channels: usize, const Frames: usize> {
    pub gain1: f64,
    pub gain2: f64,
    pub delay1: usize,
    pub delay2: usize,
    y_memory: [[f64; Frames]; Channels],
    x_memory: [[f64; Frames]; Channels],
    memory_ptr: usize,
}

impl<const Channels: usize, const Frames: usize> Comb<Channels, Frames> {
    pub fn new() -> Self {
        Self {
            gain1: 0.,
            gain2: 0.,
            delay1: 0,
            delay2: 0,
            y_memory: [[0.; Frames]; Channels],
            x_memory: [[0.; Frames]; Channels],
            memory_ptr: 0,
        }
    }
}

impl<const Channels: usize, const Frames: usize> Filter for Comb<Channels, Frames> {
    fn process(&mut self, data: &mut [Vec<f32>]) {
        assert_eq!(data.len(), Channels);
        assert!(
            self.delay1 < Frames,
            "Delay 1 must be less than frame count"
        );
        assert!(
            self.delay2 < Frames,
            "Delay 2 must be less than frame count"
        );

        for (channel_i, channel) in data.iter_mut().enumerate() {
            let mut ptr = self.memory_ptr;

            for frame in channel.iter_mut() {
                let x = *frame as f64;

                let x_read = (ptr + Frames - self.delay1) % Frames;
                let y_read = (ptr + Frames - self.delay2) % Frames;

                let mut y = unsafe {
                    let feedforward = fmul_fast(self.gain1, self.x_memory[channel_i][x_read]);
                    let feedback = fmul_fast(self.gain2, self.y_memory[channel_i][y_read]);
                    fsub_fast(fadd_fast(x, feedforward), feedback)
                };

                if !y.is_finite() {
                    y = 0.;
                }

                self.x_memory[channel_i][ptr] = x;
                self.y_memory[channel_i][ptr] = y;

                ptr += 1;
                ptr %= Frames;

                *frame = y as f32;
            }
        }

        self.memory_ptr += data.first().map(|c| c.len()).unwrap_or_default();
        self.memory_ptr %= Frames;
    }

    fn transfer_function(&self, f: f32, sample_rate: f32) -> num_complex::Complex64 {
        //    H(z) = (1 + g1 z^-M1) / 1 + g2 z^-M2
        //    let z = e^jωT
        //    H(e^jωT) = (1 + g1 e^(-M1 jωT)) / (1 + g2 e^(-M2 jωT))

        let omega = (2. * std::f32::consts::PI * f) as f64;
        let T = 1. / sample_rate as f64;

        let z1 = (-(self.delay1 as f64) * Complex64::i() * omega * T).exp();
        let z2 = (-(self.delay2 as f64) * Complex64::i() * omega * T).exp();
        let num = 1. + self.gain1 * z1;
        let denom = 1. + self.gain2 * z2;
        num / denom
    }

    fn bandwidth(&self, _sample_rate: f32) -> f32 {
        todo!()
    }

    fn process_sample(&mut self, _sample: &mut f32, _channel_i: usize) {
        todo!()
    }
}
