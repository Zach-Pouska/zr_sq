// main.rs

mod types;
mod file_parse;
mod synthesis;

use file_parse::parse_song_from_file;
use synthesis::synthesize_song;

use rodio::{source::Source};
//use hound::{WavSpec, WavWriter, SampleFormat};
use hound::{WavSpec, WavWriter};
use std::time::Duration;
use std::sync::Arc;
use std::num::{NonZeroU16, NonZeroU32};
use std::env;   // for command-line arguments
use std::process; // for exit

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---------- Parse command-line arguments ----------
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_song_file> [output_wav_file]", args[0]);
        eprintln!("   If output file is omitted, 'output.wav' is used.");
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = if args.len() >= 3 {
        &args[2]
    } else {
        "output.wav"
    };

    // ---------- Parse, synthesise, play, save ----------
    let song = parse_song_from_file(input_file)?;
    println!("Parsed song: BPM={}, Samplerate={}, Events={}",
             song.bpm, song.samplerate, song.events.len());

    let audio_buffer = synthesize_song(&song);
    let total_seconds = audio_buffer.len() as f32 / song.samplerate as f32;
    println!("Synthesized {} samples ({:.2} seconds).",
             audio_buffer.len(), total_seconds);

    play_audio(&audio_buffer, song.samplerate)?;
    save_to_wav(&audio_buffer, song.samplerate, output_file)?;

    Ok(())
}

fn play_audio(samples: &[f32], samplerate: u32) -> Result<(), Box<dyn std::error::Error>> {
    struct AudioBufferSource {
        samples: Arc<Vec<f32>>,
        pos: usize,
        samplerate: u32,
    }

    impl Iterator for AudioBufferSource {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            if self.pos < self.samples.len() {
                let sample = self.samples[self.pos];
                self.pos += 1;
                Some(sample)
            } else {
                None
            }
        }
    }

    impl Source for AudioBufferSource {
        fn current_span_len(&self) -> Option<usize> {
            Some(self.samples.len() - self.pos)
        }

        fn channels(&self) -> NonZeroU16 {
            NonZeroU16::new(1).unwrap()
        }

        fn sample_rate(&self) -> NonZeroU32 {
            NonZeroU32::new(self.samplerate).unwrap()
        }

        fn total_duration(&self) -> Option<Duration> {
            Some(Duration::from_secs_f32(
                self.samples.len() as f32 / self.samplerate as f32,
            ))
        }
    }

    let source = AudioBufferSource {
        samples: Arc::new(samples.to_vec()),
        pos: 0,
        samplerate,
    };
    
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    let mixer = handle.mixer();
    let _player = rodio::Player::connect_new(&mixer);

    println!("Beginning playback");

    let duration = source.total_duration().unwrap_or(Duration::from_secs(10));
    println!("Duration: {:?}", duration);
    //handle.mixer().add(source);
    mixer.add(source);

    std::thread::sleep(duration);

    println!("Ending playback");

    Ok(())
}

fn save_to_wav(samples: &[f32], samplerate: u32, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: samplerate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(filename, spec)?;
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }
    writer.finalize()?;
    println!("Audio saved to '{}'", filename);
    Ok(())
}
