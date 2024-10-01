use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use std::sync::mpsc::{self, Receiver, Sender};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Whisper Context Setup
    let ctx =
        WhisperContext::new_with_params("./ggml-small.bin", WhisperContextParameters::default())
            .expect("Failed to create Whisper context");
    let mut state = ctx.create_state().expect("Failed to create Whisper state");
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    params.set_n_threads(1);
    params.set_translate(true);
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(true);
    params.set_print_timestamps(false);

    // CPAL Audio Stream Setup
    let host = cpal::default_host();

    // Input device and stream setup
    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");

    let config: StreamConfig = input_device.default_input_config()?.into();

    // Create a channel to send audio data from the input stream to the output stream
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

    if let Ok(audio_data) = rx.recv() {
        // Convert f32 audio data to i16
        let audio_data_i16: Vec<i16> = audio_data
            .iter()
            .map(|&sample| (sample * i16::MAX as f32) as i16)
            .collect();

        // Convert to 16KHz mono f32 samples
        let mut inter_audio_data = vec![0.0; audio_data_i16.len()];
        whisper_rs::convert_integer_to_float_audio(&audio_data_i16, &mut inter_audio_data)
            .expect("failed to convert audio data");
        let audio_data_f32 = whisper_rs::convert_stereo_to_mono_audio(&inter_audio_data)
            .expect("failed to convert audio data");

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
            let start_timestamp = state
                .full_get_segment_t0(i)
                .expect("failed to get segment start timestamp");
            let end_timestamp = state
                .full_get_segment_t1(i)
                .expect("failed to get segment end timestamp");
            println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
        }
    }

    Ok(())
}
