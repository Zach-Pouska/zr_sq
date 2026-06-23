/// The available waveforms for a note.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

/// An individual event in the song.
#[derive(Debug, PartialEq)]
pub enum Event {
    Note {
        frequency: f32,
        duration_seconds: f32,
        volume: f32,          // 0.0 – 1.0
        waveform: Waveform,
    },
    Rest {
        duration_seconds: f32,
    },
}

/// The complete song description.
#[derive(Debug, PartialEq)]
pub struct Song {
    pub bpm: f32,
    pub samplerate: u32,
    pub events: Vec<Event>,
}
