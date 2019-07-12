use std::iter::Iterator;

#[derive(Clone)]
pub struct Oscillator {
    frequency: f32,
    sampling_rate: u32,
    index: u32,
}

impl Oscillator {
    pub fn new() -> OscillatorBuilder {
        OscillatorBuilder {
            frequency: None,
            sampling_rate: None,
            index: None,
        }
    }
}

#[derive(Clone)]
pub struct OscillatorBuilder {
    frequency: Option<f32>,
    sampling_rate: Option<u32>,
    index: Option<u32>,
}

impl OscillatorBuilder {
    pub fn frequency(&self, f: f32) -> Self {
        Self {
            frequency: Some(f),
            ..self.clone()
        }
    }

    pub fn sampling_rate(&self, sr: u32) -> Self {
        Self {
            sampling_rate: Some(sr),
            ..self.clone()
        }
    }

    pub fn index(&self, i: u32) -> Self {
        Self {
            index: Some(i),
            ..self.clone()
        }
    }

    pub fn build(&self) -> Oscillator {
        Oscillator {
            frequency: self.frequency.unwrap(),
            sampling_rate: self.sampling_rate.unwrap(),
            index: self.index.unwrap(),
        }
    }
}

impl Iterator for Oscillator {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index == self.sampling_rate {
            self.index = 0;
        }
        Some(self.index as f32 / self.sampling_rate as f32)
    }
}
