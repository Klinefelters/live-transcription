//! Run with:
//! cargo run --example microphone <model path> <duration>
//! e.g. "cargo run --example microphone /home/user/stt/model 10"
//!
//! Read the "Setup" section in the README to know how to link the vosk dynamic
//! libaries to the examples

use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    ChannelCount, SampleFormat,
};
use dasp::{sample::ToSample, Sample};
mod vosk;

fn main() {

    let record_duration = Duration::from_secs(100);

    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();


    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    // Load the Vosk model
    vosk::init_vosk(config.sample_rate().0 as f32);
    let stream = match config.sample_format() {
        SampleFormat::I8 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i8], _| recognize(data, channels),
            err_fn,
            None,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| recognize(data, channels),
            err_fn,
            None,
        ),
        SampleFormat::I32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i32], _| recognize(data, channels),
            err_fn,
            None,
        ),
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| recognize(data, channels),
            err_fn,
            None,
        ),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
    .expect("Could not build stream");

    stream.play().expect("Could not play stream");
    println!("Recording...");

    std::thread::sleep(record_duration);
    drop(stream);
}

fn recognize<T: Sample + ToSample<i16>>(
    data: &[T],
    channels: ChannelCount,
) {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    let data = if channels != 1 {
        stereo_to_mono(&data)
    } else {
        data
    };
    let recognized = vosk::recognize(&data, true);
    if let Some(text) = recognized {
        println!("Recognized: {}", text);
    }
}

pub fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );

    result
}