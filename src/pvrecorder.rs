use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use pv_recorder::{PvRecorder, PvRecorderBuilder};

static PV_RECORDER: OnceCell<PvRecorder> = OnceCell::new();
static IS_RECORDING: AtomicBool = AtomicBool::new(false);

pub fn init_microphone(device_index: i32, frame_length: u32) -> bool {
    match PV_RECORDER.get().is_none() {
        true => {
            let pv_recorder = PvRecorderBuilder::new(512)
                .device_index(device_index)
                .frame_length(frame_length as i32)
                .init();

            match pv_recorder {
                Ok(pv) => {
                    // store
                    let _ = PV_RECORDER.set(pv);

                    // success
                    true
                },
                Err(msg) => {
                    eprintln!("Failed to initialize pvPvRecorder.\neprintln details: {:?}", msg);

                    // fail
                    false
                }
            }
        },
        _ => true // already initialized
    }
}

pub fn read_microphone(_frame_buffer: &mut [i16]) {
    // ensure microphone is initialized
    if !PV_RECORDER.get().is_none() {
        // read to frame buffer
        match PV_RECORDER.get().unwrap().read() {
            Err(msg) => {
                eprintln!("Failed to read audio frame. {:?}", msg);
            },
            _ => ()
        }
    }
}

pub fn start_recording(device_index: i32, frame_length: u32) -> Result<(), ()> {
    // ensure microphone is initialized
    init_microphone(device_index, frame_length);

    // start recording
    match PV_RECORDER.get().unwrap().start() {
        Ok(_) => {
            println!("START recording from microphone ...");

            // change recording state
            IS_RECORDING.store(true, Ordering::SeqCst);

            // success
            Ok(())
        },
        Err(_msg) => {
            eprintln!("Failed to start audio recording!");

            // fail
            Err(())
        }
    }
}

// pub fn stop_recording() -> Result<(), ()> {
//     // ensure microphone is initialized & recording is in process
//     if !PV_RECORDER.get().is_none() && IS_RECORDING.load(Ordering::SeqCst) {
//         // stop recording
//         match PV_RECORDER.get().unwrap().stop() {
//             Ok(_) => {
//                 println!("STOP recording from microphone ...");

//                 // change recording state
//                 IS_RECORDING.store(false, Ordering::SeqCst);

//                 // success
//                 return Ok(())
//             },
//             Err(_msg) => {
//                 eprintln!("Failed to stop audio recording!");

//                 // fail
//                 return Err(())
//             }
//         }
//     }

//     Ok(()) // if already stopped or not yet initialized
// }