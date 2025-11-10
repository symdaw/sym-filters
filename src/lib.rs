#![feature(core_intrinsics)]
#![allow(nonstandard_style)]
#![allow(internal_features)]

mod biquad;
pub use biquad::Biquad;

pub trait Filter {
    fn process(&mut self, data: &mut [Vec<f32>]);
    fn transfer_function(&self, f: f32, sample_rate: f32) -> num_complex::Complex64;
    fn bandwidth(&self, sample_rate: f32) -> f32;

    fn amplitude_response(&self, f: f32, sample_rate: f32) -> f32 {
        self.transfer_function(f, sample_rate).norm() as f32
    }
    fn phase_response(&self, f: f32, sample_rate: f32) -> f32 {
        self.transfer_function(f, sample_rate).arg() as f32
    }
}
