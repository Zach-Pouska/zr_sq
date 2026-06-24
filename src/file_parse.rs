use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::fmt;

use crate::types::{Event, EventKind, Song};

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

/// Parse a song file. Expects optional header lines (`bpm`, `samplerate`)
/// and event lines: either `rest`/`r <dur>` or `<waveform> <freq> <dur> <vol>`.
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

        // Skip blanks and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ParseError::InvalidLine {
                line: line_num,
                message: "Empty line".to_string(),
            });
        }

        let first = parts[0].to_lowercase();

        match first.as_str() {
            "bpm" => {
                if bpm.is_some() {
                    return Err(ParseError::InvalidLine {
                        line: line_num,
                        message: "Duplicate 'bpm' line".to_string(),
                    });
                }
                if parts.len() < 2 {
                    return Err(ParseError::InvalidLine {
                        line: line_num,
                        message: "Missing value for 'bpm'".to_string(),
                    });
                }
                let val = parts[1]
                    .parse::<f32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                bpm = Some(val);
            }

            "samplerate" => {
                if samplerate.is_some() {
                    return Err(ParseError::InvalidLine {
                        line: line_num,
                        message: "Duplicate 'samplerate' line".to_string(),
                    });
                }
                if parts.len() < 2 {
                    return Err(ParseError::InvalidLine {
                        line: line_num,
                        message: "Missing value for 'samplerate'".to_string(),
                    });
                }
                let val = parts[1]
                    .parse::<u32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                samplerate = Some(val);
            }

            "rest" | "r" => {
                if parts.len() != 2 {
                    return Err(ParseError::InvalidLine {
                        line: line_num,
                        message: format!(
                            "Expected 'rest dur', got {} tokens",
                            parts.len()
                        ),
                    });
                }
                let dur = parts[1]
                    .parse::<f32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                events.push(Event {
                    duration_seconds: dur,
                    kind: EventKind::Rest,
                });
            }

            // Everything else is a note: <waveform> <freq> <dur> <vol>
            _ => {
                if parts.len() != 4 {
                    return Err(ParseError::InvalidLine {
                        line: line_num,
                        message: format!(
                            "Expected '<waveform> freq dur vol', got {} tokens",
                            parts.len()
                        ),
                    });
                }

                let freq = parts[1]
                    .parse::<f32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[1].to_string()))?;
                let dur = parts[2]
                    .parse::<f32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[2].to_string()))?;
                let vol = parts[3]
                    .parse::<f32>()
                    .map_err(|_| ParseError::InvalidNumber(parts[3].to_string()))?;

                let kind = match first.as_str() {
                    "sin" | "sine"     => EventKind::Sine    { frequency: freq, volume: vol },
                    "sqr" | "square"   => EventKind::Square  { frequency: freq, volume: vol },
                    "saw" | "sawtooth" => EventKind::Sawtooth{ frequency: freq, volume: vol },
                    "tri" | "triangle" => EventKind::Triangle{ frequency: freq, volume: vol },
                    other => return Err(ParseError::UnknownWaveform(other.to_string())),
                };

                events.push(Event {
                    duration_seconds: dur,
                    kind,
                });
            }
        }
    }

    let bpm = bpm.ok_or_else(|| ParseError::MissingField("bpm".to_string()))?;
    let samplerate =
        samplerate.ok_or_else(|| ParseError::MissingField("samplerate".to_string()))?;

    println!("{:?}", events);

    Ok(Song {
        bpm,
        samplerate,
        events,
    })
}
