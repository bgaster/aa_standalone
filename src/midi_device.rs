extern crate midir;
extern crate rimd;

use rimd::{MidiMessage, Status, STATUS_MASK};

use std::io::{stdin, stdout, Write};
use std::error::Error;
use midir::{MidiInput, Ignore, MidiInputConnection, MidiInputPort};
//use std::sync::mpsc::{Sender};
use crossbeam_channel as cb;

use crate::midi_utils::NoteSym;
use crate::comms::*;
use crate::messages::*;
use crate::utils::*;

// pub struct DeviceIn {
//     input: MidiInput,
//     port: usize,
// }

pub struct Midi {
    input_connections: Vec<MidiInputConnection<()>>,
}

// unsafe impl Send for Midi {

// }

impl Midi {
    pub fn new() -> Self {
       Self {
           input_connections: Vec::new(),
       }
    }

    pub fn get_inputs(&self) -> Result<Vec<String>> {
        MidiInput::new("midi input").map_or(
            err(), 
            |input| { 
                let mut inputs = Vec::new();
                for (i, p) in input.ports().iter().enumerate() {
                    if let Ok(name) = input.port_name(p) {
                        inputs.push(name);
                    }
                }
            ok(inputs)
        })
    }

    pub fn open_input(
        &mut self, 
        device_name: String, 
        sender: cb::Sender<MidiMessage>,
        sender_to_gui: cb::Sender<Message>) -> Result<()> {
        MidiInput::new("midi input").map_or(
            err(), 
            |input| { 
            for (i, p) in input.ports().iter().enumerate() {
                if let Ok(name) = input.port_name(p) {
                    if name == device_name {
                        let connection = input.connect(
                            p, 
                            &name, 
                            move |stamp, message, _| {
                                //println!("{}: {:?} (len = {})", stamp, message, message.len());
                                // send control messages to UI
                                if message[0] & STATUS_MASK  == 0xB0 {
                                    let controller = message[1] as Index;
                                    let data       = message[2] as i32;
                                    sender_to_gui.send(Message {
                                        id: MessageID::Control,
                                        index: controller,
                                        value: Value::VInt(data),
                                    }).unwrap();
                                }
                                else {
                                    let message = MidiMessage::from_bytes(message.iter().cloned().collect());
                                    match sender.send(message) {
                                            _ => {}
                                    }
                                }
                            }, ());
                        
                        if connection.is_err() {
                            return err();
                        }
                        self.input_connections.push(connection.unwrap());
                        return ok(());
                    }
                }
            }
            err()
        })
    }
}

