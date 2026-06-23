// file_parse.rs

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::fmt;

use crate::types::{Event, Song, Waveform};

// ---------- Error Type (manual, zero dependencies) ----------
#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    InvalidLine { line: usize, message: String },
    MissingField(String),
    UnknownWaveform(String),
    InvalidNumber(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "I/O error: {}", e),
            ParseError::InvalidLine { line, message } => {
                write!(f, "Invalid format at line {}: {}", line, message)
            }
            ParseError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParseError::UnknownWaveform(wf) => write!(f, "Unknown waveform: '{}'", wf),
            ParseError::InvalidNumber(num) => write!(f, "Invalid number: '{}'", num),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::Io(err)
    }
}

// ---------- Waveform parsing: supports both shorthands and full names ----------
impl Waveform {
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        match s.trim().to_lowercase().as_str() {
            "sin" | "sine" => Ok(Waveform::Sine),
            "sqr" | "square" => Ok(Waveform::Square),
            "saw" | "sawtooth" => Ok(Waveform::Sawtooth),
            "tri" | "triangle" => Ok(Waveform::Triangle),
            other => Err(ParseError::UnknownWaveform(other.to_string())),
        }
    }
}

/// Parse a single event line.
/// Now supports:
///   - Note:   <waveform> <freq> <dur> <vol>   (e.g., "sin 261.63 0.5 0.8")
///   - Rest:   rest <dur>
fn parse_event(line: &str, line_num: usize) -> Result<Event, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err(ParseError::InvalidLine {
            line: line_num,
            message: "Empty line".to_string(),
        });
    }

    let first = parts[0].to_lowercase();

    // Handle rest
    if first == "rest" {
        if parts.len() != 2 {
            return Err(ParseError::InvalidLine {
                line: line_num,
                message: format!("Expected 'rest dur', got {} tokens", parts.len()),
            });
        }
        let dur = parts[1].parse::<f32>()
            .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
        return Ok(Event::Rest { duration_seconds: dur });
    }

    // Otherwise, it's a note: waveform_shorthand freq dur vol
    if parts.len() != 4 {
        return Err(ParseError::InvalidLine {
            line: line_num,
            message: format!("Expected '<waveform> freq dur vol', got {} tokens", parts.len()),
        });
    }

    let waveform = Waveform::from_str(parts[0])?;
    let freq = parts[1].parse::<f32>()
        .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
    let dur = parts[2].parse::<f32>()
        .map_err(|_| ParseError::InvalidNumber(parts[2].to_string()))?;
    let vol = parts[3].parse::<f32>()
        .map_err(|_| ParseError::InvalidNumber(parts[3].to_string()))?;

    Ok(Event::Note {
        frequency: freq,
        duration_seconds: dur,
        volume: vol,
        waveform,
    })
}

pub fn parse_song_from_file<P: AsRef<Path>>(path: P) -> Result<Song, ParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut bpm: Option<f32> = None;
    let mut samplerate: Option<u32> = None;
    let mut events = Vec::new();

    for (idx, line_result) in reader.lines().enumerate() {
        let line_num = idx + 1;
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse headers if we haven't got both yet
        if bpm.is_none() || samplerate.is_none() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                match parts[0].to_lowercase().as_str() {
                    "bpm" => {
                        if bpm.is_some() {
                            return Err(ParseError::InvalidLine {
                                line: line_num,
                                message: "Duplicate 'bpm' line".to_string(),
                            });
                        }
                        let val = parts[1].parse::<f32>()
                            .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                        bpm = Some(val);
                        continue;
                    }
                    "samplerate" => {
                        if samplerate.is_some() {
                            return Err(ParseError::InvalidLine {
                                line: line_num,
                                message: "Duplicate 'samplerate' line".to_string(),
                            });
                        }
                        let val = parts[1].parse::<u32>()
                            .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                        samplerate = Some(val);
                        continue;
                    }
                    _ => {} // not a header, fall through to event parsing
                }
            }
        }

        // Parse as an event
        let event = parse_event(trimmed, line_num)?;
        events.push(event);
    }

    let bpm = bpm.ok_or_else(|| ParseError::MissingField("bpm".to_string()))?;
    let samplerate = samplerate.ok_or_else(|| ParseError::MissingField("samplerate".to_string()))?;

    Ok(Song { bpm, samplerate, events })
}

