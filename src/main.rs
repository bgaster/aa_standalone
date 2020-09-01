
mod gui;

mod standalone;
use standalone::*;

mod comms;
mod messages;
mod bundle;
mod utils;
mod midi_utils;
mod midi_device;

//-----------------------------------------------------------------------------

fn main() {
    Standalone::new("http://127.0.0.1:8081")
    //Standalone::new("https://bgaster.github.io/audio_anywhere/")
        .unwrap()
        .run()
        .unwrap();
}