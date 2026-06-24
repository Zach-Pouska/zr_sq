/// The part of an event that varies by type.
#[derive(Debug, PartialEq)]
pub enum EventKind {
    Rest,
    Sine { frequency: f32, volume: f32 },
    Square { frequency: f32, volume: f32 },
    Sawtooth { frequency: f32, volume: f32 },
    Triangle { frequency: f32, volume: f32 },
}

#[derive(Debug, PartialEq)]
pub struct Event {
    pub duration_seconds: f32,
    pub kind: EventKind,
}

/// The complete song description.
#[derive(Debug, PartialEq)]
pub struct Song {
    pub bpm: f32,
    pub samplerate: u32,
    pub events: Vec<Event>,
}

