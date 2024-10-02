use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use std::sync::mpsc::{self, Receiver, Sender};
// use std::thread;
// use std::time::Duration;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Whisper Context Setup
    let ctx =
        WhisperContext::new_with_params("./ggml-small.bin", WhisperContextParameters::default())
            .expect("Failed to create Whisper context");
    let mut state = ctx.create_state().expect("Failed to create Whisper state");

    // CPAL Audio Stream Setup
    let host = cpal::default_host();

    // Input device and stream setup
    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");

    let config: StreamConfig = input_device.default_input_config()?.into();

    // Create a channel to send audio data from the input stream to the main thread
    let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

    // Build the input stream
    let input_stream = input_device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Send the audio data to the main thread
                if let Err(err) = tx.send(data.to_vec()) {
                    eprintln!("Failed to send audio data: {}", err);
                }
            },
            move |err| {
                eprintln!("Error occurred on input stream: {}", err);
            },
            None,
        )
        .expect("Failed to build input stream");

    // Start the input stream
    input_stream.play().expect("Failed to start input stream");

    // Buffer to accumulate audio data
    let mut audio_buffer: Vec<f32> = Vec::new();
    let chunk_duration = 1.0; // collect 1 second of audio before transcription
    let sample_rate = config.sample_rate.0 as usize;
    let channels = config.channels as usize;
    let samples_per_chunk = (sample_rate * channels) as usize;

    // Loop to continuously transcribe audio in real-time
    loop {
        if let Ok(audio_data) = rx.recv() {
            audio_buffer.extend(audio_data);

            // Check if we've accumulated enough audio for a chunk
            if audio_buffer.len() >= samples_per_chunk * chunk_duration as usize {
                let audio_data_f32: Vec<f32> = audio_buffer.clone();
                audio_buffer.clear(); // Clear the buffer for the next chunk
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

                params.set_n_threads(1);
                params.set_translate(true);
                params.set_language(Some("en"));
                params.set_print_special(true);
                params.set_print_progress(true);
                params.set_print_realtime(true);
                params.set_print_timestamps(true);

                // Run the model
                state
                    .full(params, &audio_data_f32[..])
                    .expect("failed to run model");

                // Fetch the results
                let num_segments = state
                    .full_n_segments()
                    .expect("failed to get number of segments");
                for i in 0..num_segments {
                    let segment = state
                        .full_get_segment_text(i)
                        .expect("failed to get segment");
                    println!("{}", segment);
                }
            }
        }

        // Sleep for a short duration to avoid busy waiting
        // thread::sleep(Duration::from_millis(10));
    }
}
