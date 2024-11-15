use pv_recorder::PvRecorderBuilder;
use vosk::{DecodingState, Model, Recognizer};

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::thread;
use std::time::Duration;

fn load_word_pool(file_path: &str) -> Result<HashSet<String>, io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    // Read words line by line and insert into a HashSet
    let word_pool: HashSet<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();

    Ok(word_pool)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the word pool from an external file
    let pool_of_words = load_word_pool("src/word_pool.txt")?;


    // Initialize VOSK model and recognizer
    let model = Model::new("./model").expect("Failed to load model");
    let mut recognizer = Recognizer::new(&model, 16000.0).expect("Failed to create recognizer");

    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let recorder = PvRecorderBuilder::new(512).init()?;
    recorder.start()?;
    let mut last_transcription = String::new();

    // Main loop for capturing audio and transcribing
    while recorder.is_recording() {
        let frame = recorder.read()?;

        let state = recognizer.accept_waveform(&frame);

        match state {
            DecodingState::Running => {
                let partial = recognizer.partial_result().partial.into();
                if partial != last_transcription {
                    println!("Partial: {}", partial);
                    last_transcription = partial;
                }
            }
            DecodingState::Finalized => {
                let result: String = recognizer.result()
                    .multiple()
                    .unwrap()
                    .alternatives
                    .first()
                    .unwrap()
                    .text
                    .into();

                println!("Final: {}", result);

                // Split the result into individual words and process
                let recognized_words: Vec<&str> = result.split_whitespace().collect();
                let total_words = recognized_words.len();
                let mut total_correct = 0;

                // Count correct words
                for word in &recognized_words {
                    if pool_of_words.contains(*word) {
                        total_correct += 1;
                    }
                }

                // Calculate Word Accuracy Rate (WAR) for the current finalized result
                if total_words > 0 {
                    let war = total_correct as f32 / total_words as f32;
                    println!("Word Accuracy Rate (WAR) for current result: {:.2}%", war * 100.0);
                } else {
                    println!("No words recognized in this finalized result.");
                }

                // Stop recording if the word "stop" is detected
                if result.contains("stop") {
                    recorder.stop()?;
                }
            }
            DecodingState::Failed => {
                println!("Failed to decode audio");
            }
        }

        // Sleep for a bit to prevent high CPU usage
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
