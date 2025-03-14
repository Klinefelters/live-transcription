// This code was adapted from https://github.com/Priler/jarvis/blob/master/app/src/stt/vosk.rs
// Curtosy of Abraham Tugalov (aka Priler)
use once_cell::sync::OnceCell;
use vosk::{DecodingState, Model, Recognizer};
use std::sync::Mutex;
static MODEL: OnceCell<Model> = OnceCell::new();
static RECOGNIZER: OnceCell<Mutex<Recognizer>> = OnceCell::new();
pub fn init_vosk() {
    if !RECOGNIZER.get().is_none() {return;} // already initialized
    let model = Model::new("./model").unwrap();
    let mut recognizer = Recognizer::new(&model, 16000.0).unwrap();
    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);
    let _ = MODEL.set(model);
    let _ = RECOGNIZER.set(Mutex::new(recognizer));
}
pub fn recognize(data: &[i16], include_partial: bool) -> Option<String> {
    let state = RECOGNIZER.get().unwrap().lock().unwrap().accept_waveform(data);
    match state {
        DecodingState::Running => {
            if include_partial {
                Some(RECOGNIZER.get().unwrap().lock().unwrap().partial_result().partial.into())
            } else {
                None
            }
        }
        DecodingState::Finalized => {
            // Result will always be multiple because we called set_max_alternatives
            Some(
                RECOGNIZER.get().unwrap().lock().unwrap()
                    .result()
                    .multiple()
                    .unwrap()
                    .alternatives
                    .first()
                    .unwrap()
                    .text
                    .into(),
            )
        }
        DecodingState::Failed => None,
    }
}