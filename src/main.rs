mod vosk;
use pv_recorder::PvRecorderBuilder;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize VOSK model and recognizer
    vosk::init_vosk();
    let frame_length = 512;
    let recorder = PvRecorderBuilder::new(frame_length).init()?;
    recorder.start()?;

    // Main loop for capturing audio and transcribing
    while recorder.is_recording() {
        let frame = recorder.read()?;

        // Pass the frame data to Vosk for transcription
        if let Some(transcription) = vosk::recognize(&frame, true) {
            println!("{}", transcription);
        }

        // Sleep for a bit to prevent high CPU usage
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())

    // Stop the recording (optional, since this example loops forever)
    // pvrecorder::stop_recording().unwrap();
}
