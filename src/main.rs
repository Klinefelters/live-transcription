use pv_recorder::PvRecorderBuilder;
use vosk::{DecodingState, Model, Recognizer};

use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                    .multiple().
                    unwrap().
                    alternatives.
                    first().
                    unwrap().
                    text.
                    into();
                
                println!("Final: {}", result);
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
