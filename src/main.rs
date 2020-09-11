//! 
//! main for standalone app
//! Copyright: Benedict R. Gaster
//! 
use clap::Clap;


use anyhow::{anyhow, Result};

mod gui;

mod standalone;
use standalone::*;

mod comms;
mod messages;
mod bundle;
mod utils;
mod midi_utils;
mod midi_device;

use crate::midi_device::*;

//-----------------------------------------------------------------------------

#[derive(Clap)]
#[clap(version = "0.1", author = "Benedict R. Gaster <benedict.gaster@uwe.ac.uk>")]
struct Opts {
    /// URL for AA server
    #[clap(short, long, default_value = "http://127.0.0.1")]
    url: String,
    /// Optional port for AA server
    #[clap(short, long)]
    port: Option<String>,
    /// Optional MIDI input device to use
    #[clap(short, long)]
    midi_device: Option<String>,
    #[clap(short, long)]
    list_midi_devices: bool,
}   

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    if opts.list_midi_devices {
        let midi = Midi::new();
        let midi_inputs = midi.get_inputs().map_err(|_| anyhow!("Standalone run failed"))?;
        println!("Available MIDI devices (choose with option -m):");
        for md in midi_inputs.iter() {
            println!("{}", md);
        }
        return Ok(());
    }

    let url = 
        if let Some(p) = opts.port {
            [&opts.url, ":", &p].join("")
        }
        else {
            opts.url.clone()
        };

    let standalone = Standalone::new(&url, opts.midi_device).map_err(|_| anyhow!("Failed to create standalone"))?;
    standalone.run().map_err(|_| anyhow!("Standalone run failed"))?;
    
    Ok(())
}
