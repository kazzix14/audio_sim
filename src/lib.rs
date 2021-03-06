#[macro_use]
extern crate glium;

pub mod gui;
pub mod oscillator;
pub mod wave_simulator;

pub const SIZE: usize = 96;
pub const NUM_THREADS: usize = 2;
pub const SLEEP_TIME: u64 = 0;
