use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;

mod vosk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vosk::init_vosk();

    let process_audio = |data: &[i16], _: &cpal::InputCallbackInfo| {
        if let Some(result) = vosk::recognize(data, true) {
            if result.trim().is_empty() {
                return;
            }
            println!("Recognized: {}", result);
        }
    };

    // CPAL Audio Stream Setup
    let host = cpal::default_host();

    // Input device and stream setup
    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");

    let supported_configs_range = input_device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .filter(|config| config.sample_format() == SampleFormat::I16)
        .map(|config| config.with_max_sample_rate())
        .next()
        .expect("No supported config with sample format I16");

    println!("Using config: {:?}", supported_config);

    let err_fn = |err| eprintln!("Error occurred on input stream: {}", err);

    // Build the input stream
    let input_stream = input_device.build_input_stream(&supported_config.into(), process_audio, err_fn, None)?;

    // Start the streams
    input_stream.play().expect("Failed to start input stream");

    std::thread::sleep(std::time::Duration::from_secs(60)); // Keep the stream alive for 60 seconds
    // Cleanup and exit
    drop(input_stream);

    Ok(())
}