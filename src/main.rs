mod vosk;
mod pvrecorder;

use std::thread;
use std::time::Duration;

fn main() {
    // Initialize VOSK model and recognizer
    vosk::init_vosk();

    // Device index for the microphone (you can adjust this)
    let device_index: i32 = 0;
    // Frame length, typically 512 or 1024 (depends on your mic)
    let frame_length: u32 = 512;

    // Initialize microphone
    if !pvrecorder::init_microphone(device_index, frame_length) {
        eprintln!("Failed to initialize microphone");
        return;
    }

    // Start recording from the microphone
    if let Err(_) = pvrecorder::start_recording(device_index, frame_length) {
        eprintln!("Failed to start recording");
        return;
    }

    // Buffer for the audio data
    let mut frame_buffer = vec![0i16; frame_length as usize];

    // Main loop for capturing audio and transcribing
    loop {
        // Read audio frame from microphone
        pvrecorder::read_microphone(&mut frame_buffer);

        // Pass the frame data to Vosk for transcription
        if let Some(transcription) = vosk::recognize(&frame_buffer, true) {
            println!("{}", transcription);
        }

        // Sleep for a bit to prevent high CPU usage
        thread::sleep(Duration::from_millis(10));
    }

    // Stop the recording (optional, since this example loops forever)
    // pvrecorder::stop_recording().unwrap();
}
