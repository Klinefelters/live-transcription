use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use std::sync::mpsc::{self, Receiver, Sender};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CPAL Audio Stream Setup
    let host = cpal::default_host();

    // Input device and stream setup
    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");

    // Output device and stream setup
    let output_device = host
        .default_output_device()
        .expect("Failed to get default output device");

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

    // Build the output stream
    let output_stream = output_device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Receive the audio data from the input stream
                if let Ok(input_data) = rx.try_recv() {
                    // Copy the input data to the output buffer
                    for (out_sample, in_sample) in data.iter_mut().zip(input_data.iter()) {
                        *out_sample = *in_sample;
                    }
                }
            },
            move |err| {
                eprintln!("Error occurred on output stream: {}", err);
            },
            None,
        )
        .expect("Failed to build output stream");

    // Start the streams
    input_stream.play().expect("Failed to start input stream");
    output_stream.play().expect("Failed to start output stream");

    println!("Audio streaming started. Press Ctrl+C to stop.");

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
