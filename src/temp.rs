use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use vosk::{Model, Recognizer};

fn main() {
    // Vosk Setup
    let model_path = "./model";
    let model = Model::new(model_path).expect("Failed to create model");
    let mut recognizer = Recognizer::new(&model, 16000.0).expect("Failed to create recognizer");

    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    // CPAL Audio Stream Setup
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Failed to get default input device");

    let mut configs_range = device
        .supported_input_configs()
        .expect("Failed to get supported input configs");
    let config = configs_range
        .next()
        .expect("Failed to get next config")
        .with_max_sample_rate()
        .config();

    // Create a channel to send audio data from the stream to the main thread
    let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Send the audio data to the main thread
                if let Err(err) = tx.send(data.to_vec()) {
                    eprintln!("Failed to send audio data: {}", err);
                }
            },
            move |err| {
                eprintln!("Error occurred on stream: {}", err);
            },
            None,
        )
        .expect("Failed to build input stream");

    // Start the stream
    stream.play().expect("Failed to start stream");

    println!("Stream started. Transcribing for 10 seconds...");

    // Handle the audio data in the main thread
    thread::spawn(move || {
        let mut buffer: Vec<i16> = Vec::new();
        while let Ok(data) = rx.recv() {
            // Convert f32 data to i16 and append to buffer
            buffer.extend(data.iter().map(|&sample| (sample * i16::MAX as f32) as i16));

            // If buffer size is large enough, process it with the recognizer
            if buffer.len() >= 16000 {
                recognizer.accept_waveform(&buffer);
                // buffer.clear();

                // Get the partial result
                let partial_result = recognizer.partial_result();

                // Extract and print the recognized words
                let words: Vec<&str> = partial_result
                    .partial_result
                    .iter()
                    .map(|word| word.word)
                    .collect();
                println!("{:?}", words.join(" "));
                print!("{:?}", recognizer.partial_result());
            }
        }
        print!("{:?}", recognizer.final_result());
    });

    // Keep the stream running for a certain duration
    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("Transcribing finished.");
}
