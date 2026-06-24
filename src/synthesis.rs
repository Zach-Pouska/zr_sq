// synthesis.rs

use std::f32::consts::PI;
use crate::types::{Song, EventKind};



/// Generate a vector of samples for a single note.
fn generate_note_samples(
    frequency: f32,
    duration_seconds: f32,
    volume: f32,
    samplerate: u32,
    waveform_sample: impl Fn(f32) -> f32,
) -> Vec<f32> {
    let num_samples = (duration_seconds * samplerate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / samplerate as f32;
        let phase = frequency * t;
        let sample = waveform_sample(phase) * volume;
        samples.push(sample);
    }
    samples
}

/// Synthesize the entire song into a mono audio buffer (f32 samples).
pub fn synthesize_song(song: &Song) -> Vec<f32> {
    let mut current_time = 0; // sample index where the next note will be mixed in
    let mut buffer: Vec<f32> = Vec::new();

    for event in &song.events {
        // Duration is shared – no need to pattern match to get it
        let dur_samples = (event.duration_seconds * song.samplerate as f32) as usize;

        // Ensure the buffer is large enough for this event
        match event.kind {
            EventKind::Rest => {
                // Rests append silence at the end of the buffer and advance the cursor
                let new_len = buffer.len() + dur_samples;
                buffer.resize(new_len, 0.0);
                current_time += dur_samples;
            }
            _ => {
                // Notes are inserted at current_time, possibly extending the buffer
                let required_len = current_time + dur_samples;
                if required_len > buffer.len() {
                    buffer.resize(required_len, 0.0);
                }
                // current_time stays the same for overlapping notes
            }
        }

        // Generate and mix the actual waveform samples
        match &event.kind {
            EventKind::Sine { frequency, volume } => {
                let samples = generate_note_samples(
                    *frequency,
                    event.duration_seconds,
                    *volume,
                    song.samplerate,
                    |phase| (2.0 * PI * phase).sin(),
                );
                for (i, s) in samples.iter().enumerate() {
                    buffer[current_time + i] += s;
                }
            }
            EventKind::Square { frequency, volume } => {
                let samples = generate_note_samples(
                    *frequency,
                    event.duration_seconds,
                    *volume,
                    song.samplerate,
                    |phase| {
                        if (2.0 * PI * phase).sin() >= 0.0 { 1.0 } else { -1.0 }
                    },
                );
                for (i, s) in samples.iter().enumerate() {
                    buffer[current_time + i] += s;
                }
            }
            EventKind::Sawtooth { frequency, volume } => {
                let samples = generate_note_samples(
                    *frequency,
                    event.duration_seconds,
                    *volume,
                    song.samplerate,
                    |phase| 2.0 * (phase - phase.floor()) - 1.0,
                );
                for (i, s) in samples.iter().enumerate() {
                    buffer[current_time + i] += s;
                }
            }
            EventKind::Triangle { frequency, volume } => {
                let samples = generate_note_samples(
                    *frequency,
                    event.duration_seconds,
                    *volume,
                    song.samplerate,
                    |phase| {
                        2.0 * (2.0 * (phase - phase.floor()) - 1.0).abs() - 1.0
                    },
                );
                for (i, s) in samples.iter().enumerate() {
                    buffer[current_time + i] += s;
                }
            }
            EventKind::Rest => {
                // Silence is already in place from the buffer extension above
            }
        }
    }

    buffer
}
