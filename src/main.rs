use pv_recorder::PvRecorderBuilder;
use whisper_rs::{
    FullParams, 
    SamplingStrategy, 
    WhisperContext, 
    WhisperContextParameters
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Whisper Context Setup
    let ctx =
        WhisperContext::new_with_params("./ggml-small.bin", WhisperContextParameters::default())
            .expect("Failed to create Whisper context");
    let mut state = ctx.create_state().expect("Failed to create Whisper state");

    // Initialize the audio recorder
    let recorder = PvRecorderBuilder::new(512).init()?;
    recorder.start()?;

    // Loop to continuously transcribe audio in real-time
    while recorder.is_recording() {
        let frame = recorder.read()?;

        let f32_frame = frame
            .iter()
            .map(|&x| x as f32 / i16::MAX as f32)
            .collect::<Vec<f32>>();
            
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
            .full(params, &f32_frame)
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
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
