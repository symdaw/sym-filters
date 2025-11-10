use std::{
    f32,
    intrinsics::{fadd_fast, fmul_fast, fsub_fast},
};

use num_complex::Complex64;

use crate::Filter;

#[derive(Default)]
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
            // Trying to encourage using registers.
            // Sets back at end of function.
            // Maybe unnecessary.
            let mut x_memory = self.x_memory[channel_i];
            let mut y_memory = self.y_memory[channel_i];

            let b0 = self.b0 / self.a0;
            let b1 = self.b1 / self.a0;
            let b2 = self.b2 / self.a0;
            let a1 = self.a1 / self.a0;
            let a2 = self.a2 / self.a0;

            for frame in channel.iter_mut() {
                let x = *frame as f64;

                // Lame version:
                // let y = b0 * x + b1 * x_memory[0] + b2 * x_memory[1]
                //     - a1 * y_memory[0]
                //     - a2 * y_memory[1];

                let y = unsafe {
                    let preset = fmul_fast(b0, x);
                    let feedforward =
                        fadd_fast(fmul_fast(b1, x_memory[0]), fmul_fast(b2, x_memory[1]));
                    let feedback =
                        fadd_fast(fmul_fast(a1, y_memory[0]), fmul_fast(a2, y_memory[1]));

                    fsub_fast(fadd_fast(preset, feedforward), feedback)
                };

                y_memory[1] = y_memory[0];
                x_memory[1] = x_memory[0];
                y_memory[0] = y;
                x_memory[0] = x;

                *frame = y as f32;
            }

            for mem in x_memory.iter_mut() {
                if !mem.is_finite() || mem.is_subnormal() {
                    *mem = 0.;
                }
            }
            for mem in y_memory.iter_mut() {
                if !mem.is_finite() || mem.is_subnormal() {
                    *mem = 0.;
                }
            }

            self.x_memory[channel_i] = x_memory;
            self.y_memory[channel_i] = y_memory;
        }
    }

    fn transfer_function(&self, f: f32, sample_rate: f32) -> Complex64 {
        //    H(z) = (b0 + b1 z^-1 + b2 z^-2) / (a0 + a1 z^-1 + a2 z^-2)
        //    let z = e^jωT
        //    H(e^jωT) = (b0 + b1 e^-jωT + b2 e^-2jωT) / (a0 + a1 e^-jωT + a2 e^-2jωT)
        //    G(ω) = |H(e^jωT)|

        let omega = (2. * std::f32::consts::PI * f / sample_rate) as f64;
        let z1 = (-Complex64::i() * omega).exp();
        let z2 = (-2. * Complex64::i() * omega).exp();
        let num = self.b0 + self.b1 * z1 + self.b2 * z2;
        let denom = self.a0 + self.a1 * z1 + self.a2 * z2;
        num / denom
    }

    fn bandwidth(&self, sample_rate: f32) -> f32 {
        todo!()
    }
}

impl Biquad {
    pub fn new() -> Self {
        Self::default()
    }

    // All of these are from the EQ cookbook (https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html)
    // They are on biliear transformations of prototype analog filters.

    pub fn lpf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = (1. - omega_0.cos()) / 2.;
        self.b1 = 1. - omega_0.cos();
        self.b2 = (1. - omega_0.cos()) / 2.;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
    }

    pub fn hpf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = (1. + omega_0.cos()) / 2.;
        self.b1 = -(1. + omega_0.cos());
        self.b2 = (1. + omega_0.cos()) / 2.;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
    }

    pub fn bpf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = q as f64 * alpha;
        self.b1 = 0.;
        self.b2 = -q as f64 * alpha;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
    }

    pub fn notch(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = 1.;
        self.b1 = -2. * omega_0.cos();
        self.b2 = 1.;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
    }

    pub fn apf(&mut self, cutoff: f32, q: f32, sample_rate: f32) {
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);
        self.b0 = 1. - alpha;
        self.b1 = -2. * omega_0.cos();
        self.b2 = 1. + alpha;
        self.a0 = 1. + alpha;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha;
    }

    pub fn bell(&mut self, cutoff: f32, q: f32, gain: f32, sample_rate: f32) {
        let A = 10f32.powf(gain / 40.) as f64;
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);

        self.b0 = 1. + alpha * A;
        self.b1 = -2. * omega_0.cos();
        self.b2 = 1. - alpha * A;
        self.a0 = 1. + alpha / A;
        self.a1 = -2. * omega_0.cos();
        self.a2 = 1. - alpha / A;
    }

    pub fn low_shelf(&mut self, cutoff: f32, q: f32, gain: f32, sample_rate: f32) {
        let A = 10f32.powf(gain / 40.) as f64;
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);

        self.b0 = A * ((A + 1.) - (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha);
        self.b1 = 2. * A * ((A - 1.) - (A + 1.) * omega_0.cos());
        self.b2 = A * ((A + 1.) - (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha);

        self.a0 = (A + 1.) - (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha;
        self.a1 = -2. * ((A - 1.) - (A + 1.) * omega_0.cos());
        self.a2 = (A + 1.) - (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha;
    }

    pub fn high_shelf(&mut self, cutoff: f32, q: f32, gain: f32, sample_rate: f32) {
        let A = 10f32.powf(gain / 40.) as f64;
        let omega_0 = (2. * f32::consts::PI * cutoff / sample_rate) as f64;
        let alpha = omega_0.sin() / (2. * q as f64);

        self.b0 = A * ((A + 1.) + (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha);
        self.b1 = 2. * A * ((A - 1.) + (A + 1.) * omega_0.cos());
        self.b2 = A * ((A + 1.) + (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha);

        self.a0 = (A + 1.) - (A - 1.) * omega_0.cos() + 2. * A.sqrt() * alpha;
        self.a1 = -2. * ((A - 1.) - (A + 1.) * omega_0.cos());
        self.a2 = (A + 1.) - (A - 1.) * omega_0.cos() - 2. * A.sqrt() * alpha;
    }
}
