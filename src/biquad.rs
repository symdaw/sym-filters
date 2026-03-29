use std::{
    f32,
    intrinsics::{fadd_fast, fmul_fast, fsub_fast},
};

use num_complex::Complex64;

use crate::Filter;

#[derive(Default, Clone)]
pub struct Biquad {
    y_memory: [[f64; 2]; 16],
    x_memory: [[f64; 2]; 16],
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}

impl Filter for Biquad {
    fn process(&mut self, data: &mut [Vec<f32>]) {
        for (channel_i, channel) in data.iter_mut().enumerate() {
            for frame in channel.iter_mut() {
                self.process_sample(frame, channel_i);
            }
            self.cleanup(channel_i);
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, sample: &mut f32, channel_i: usize) {
        let x = *sample as f64;

        let mut y = unsafe {
            let present = fmul_fast(self.b0, x);
            let feedforward = fadd_fast(
                fmul_fast(self.b1, self.x_memory[channel_i][0]),
                fmul_fast(self.b2, self.x_memory[channel_i][1]),
            );
            let feedback = fadd_fast(
                fmul_fast(self.a1, self.y_memory[channel_i][0]),
                fmul_fast(self.a2, self.y_memory[channel_i][1]),
            );

            fsub_fast(fadd_fast(present, feedforward), feedback)
        };

        if !y.is_finite() {
            y = 0.;
        }

        self.y_memory[channel_i][1] = self.y_memory[channel_i][0];
        self.x_memory[channel_i][1] = self.x_memory[channel_i][0];
        self.y_memory[channel_i][0] = y;
        self.x_memory[channel_i][0] = x;

        *sample = y as f32;
    }

    fn transfer_function(&self, f: f32, sample_rate: f32) -> Complex64 {
        //    H(z) = (b0 + b1 z^-1 + b2 z^-2) / (a0 + a1 z^-1 + a2 z^-2)
        //    let z = e^jωT
        //    H(e^jωT) = (b0 + b1 e^-jωT + b2 e^-2jωT) / (a0 + a1 e^-jωT + a2 e^-2jωT)

        let omega = (2. * std::f32::consts::PI * f) as f64;
        let T = 1. / sample_rate as f64;

        let z1 = (-Complex64::i() * omega * T).exp();
        let z2 = (-2. * Complex64::i() * omega * T).exp();
        let num = self.b0 + self.b1 * z1 + self.b2 * z2;
        let denom = self.a0 + self.a1 * z1 + self.a2 * z2;
        num / denom
    }

    fn bandwidth(&self, _sample_rate: f32) -> f32 {
        todo!()
    }
}

impl Biquad {
    pub fn new() -> Self {
        Self::default()
    }

    fn normalize(&mut self) {
        self.b0 /= self.a0;
        self.b1 /= self.a0;
        self.b2 /= self.a0;
        self.a1 /= self.a0;
        self.a2 /= self.a0;
        self.a0 = 1.;
    }

    fn cleanup(&mut self, channel_i: usize) {
        for mem in self.x_memory[channel_i].iter_mut() {
            if !mem.is_finite() || mem.is_subnormal() {
                *mem = 0.;
            }
        }
        for mem in self.y_memory[channel_i].iter_mut() {
            if !mem.is_finite() || mem.is_subnormal() {
                *mem = 0.;
            }
        }
    }

    // All of these are from the EQ cookbook (https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html)
    // They are pre-warped biliear transformations of prototype analog filters.

    pub fn lpf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = (1. - omega_0.cos()) / 2.;
        self.b1 = 1. - omega_0.cos();
        self.b2 = (1. - omega_0.cos()) / 2.;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
        self.normalize();
    }

    pub fn hpf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = (1. + omega_0.cos()) / 2.;
        self.b1 = -(1. + omega_0.cos());
        self.b2 = (1. + omega_0.cos()) / 2.;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
        self.normalize();
    }

    pub fn bpf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = q as f64 * alpha;
        self.b1 = 0.;
        self.b2 = -q as f64 * alpha;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
        self.normalize();
    }

    pub fn notch(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = 1.;
        self.b1 = -2. * omega_0.cos();
        self.b2 = 1.;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
        self.normalize();
    }

    pub fn apf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = 1. - alpha;
        self.b1 = -2. * omega_0.cos();
        self.b2 = 1. + alpha;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
        self.normalize();
    }

    pub fn bell(&mut self, cutoff: f32, q: f32, gain: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let A = 10f32.powf(gain / 40.) as f64;
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);

        self.b0 = 1. + alpha * A;
        self.b1 = -2. * omega_0.cos();
        self.b2 = 1. - alpha * A;
        self.a0 = 1. + alpha / A;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha / A;
        self.normalize();
    }

    pub fn low_shelf(&mut self, cutoff: f32, q: f32, gain: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let A = 10f32.powf(gain / 40.) as f64;
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);

        self.b0 = A * ((A + 1.) - (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha);
        self.b1 = 2. * A * ((A - 1.) - (A + 1.) * omega_0.cos());
        self.b2 = A * ((A + 1.) - (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha);

        self.a0 = (A + 1.) + (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha;
        self.a1 = -2. * ((A - 1.) + (A + 1.) * omega_0.cos());
        self.a2 = (A + 1.) + (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha;
        self.normalize();
    }

    pub fn high_shelf(&mut self, cutoff: f32, q: f32, gain: f32, sample_rate: f32) {
        if cutoff >= sample_rate / 2. {
            return;
        }

        let A = 10f32.powf(gain / 40.) as f64;
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);

        self.b0 = A * ((A + 1.) + (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha);
        self.b1 = -2. * A * ((A - 1.) + (A + 1.) * omega_0.cos());
        self.b2 = A * ((A + 1.) + (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha);

        self.a0 = (A + 1.) - (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha;
        self.a1 = 2. * ((A - 1.) - (A + 1.) * omega_0.cos());
        self.a2 = (A + 1.) - (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha;
        self.normalize();
    }
}
