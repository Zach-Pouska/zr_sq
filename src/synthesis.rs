// synthesis.rs

use std::f32::consts::PI;
use crate::types::{Song, Event, Waveform};

/// Generate a single sample of a given waveform at a given phase (0..1).
fn sample_waveform(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (2.0 * PI * phase).sin(),
        Waveform::Square => if (2.0 * PI * phase).sin() >= 0.0 { 1.0 } else { -1.0 },
        Waveform::Sawtooth => 2.0 * (phase - phase.floor()) - 1.0,
        Waveform::Triangle => 2.0 * (2.0 * (phase - phase.floor()) - 1.0).abs() - 1.0,
    }
}

/// Generate a vector of samples for a single note.
fn generate_note_samples(
    frequency: f32,
    duration_seconds: f32,
    volume: f32,
    waveform: Waveform,
    samplerate: u32,
) -> Vec<f32> {
    let num_samples = (duration_seconds * samplerate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / samplerate as f32;
        let phase = frequency * t;
        let sample = sample_waveform(waveform, phase) * volume;
        samples.push(sample);
    }
    samples
}

/// Synthesize the entire song into a mono audio buffer (f32 samples).
pub fn synthesize_song(song: &Song) -> Vec<f32> {
    let mut current_time = 0; // integer value for the current sample - tracks when new sounds are
    let mut buffer = Vec::new();

    for event in &song.events {
        match event {
            Event::Note { frequency, duration_seconds, volume, waveform } => {
                let samples = generate_note_samples(
                    *frequency,
                    *duration_seconds,
                    *volume,
                    *waveform,
                    song.samplerate,
                );
                let length = samples.len();
                let bufferlength = buffer.len();
                if length+current_time > bufferlength {
                    buffer.extend(vec![0.0; current_time + length -bufferlength]); // extend by number of necessary samples,
                }
                // and then additively introduce the sound with a slice
                for i in 0..length {
                    buffer[i+current_time] = buffer[i+current_time] + samples[i];
                }
            }
            Event::Rest { duration_seconds } => {
                let silence_samples = (*duration_seconds * song.samplerate as f32) as usize;
                current_time += silence_samples;
                buffer.extend(vec![0.0; silence_samples]);
            }
        }
    }
    buffer
}

