use crate::Filter;

#[derive(Clone, Default)]
pub struct Driven<F: Filter> {
    pub inner_filter: F,
    pub drive: f32,
    pub prev_samples: [f32; 16],
    pub input_volume: [f32; 16],
}

impl<F: Filter> Filter for Driven<F> {
    fn process_sample(&mut self, sample: &mut f32, channel_i: usize) {
        self.input_volume[channel_i] = crate::utils::lerp(self.input_volume[channel_i], sample.abs(), 0.01);

        let mut feedback_gain = 0.5;

        const THRESHOLD: f32 = 0.05;
        if self.input_volume[channel_i] < THRESHOLD {
            feedback_gain *= self.input_volume[channel_i] / THRESHOLD;
        }

        *sample = (*sample + self.prev_samples[channel_i] * feedback_gain).tanh();
        self.inner_filter.process_sample(sample, channel_i);
        self.prev_samples[channel_i] = *sample;
    }

    fn bandwidth(&self, _sample_rate: f32) -> f32 {
        todo!()
    }
}
