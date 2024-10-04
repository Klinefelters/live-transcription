mod vosk;
use pv_recorder::PvRecorderBuilder;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize VOSK model and recognizer
    vosk::init_vosk();
    let recorder = PvRecorderBuilder::new(512).init()?;
    recorder.start()?;

    // Main loop for capturing audio and transcribing
    while recorder.is_recording() {
        let frame = recorder.read()?;

        // Pass the frame data to Vosk for transcription
        if let Some(transcription) = vosk::recognize(&frame, true) {
            if transcription.is_empty() {
                continue;
            }
            println!("{}", transcription);
            if transcription.contains("stop") {
                recorder.stop()?;
            }
        }

        // Sleep for a bit to prevent high CPU usage
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
