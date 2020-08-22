//! 
//! Wasmtime implementation of standalone app
//! Copyright: Benedict R. Gaster
//! 
#![allow(dead_code)]

use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;

extern crate crossbeam_channel;
use crossbeam_channel as cb;

use crate::gui::*;
//use crate::audio_anywhere_wasmtime::*;
use aa_wasmtime::*;
use crate::messages::*;
use crate::comms::*;
use crate::utils::*;
use crate::bundle::*;

extern crate portaudio;
use portaudio as pa;

use crate::midi_device::*;
use rimd::{MidiMessage, Status};

/// Wasmtime based Standalone Audio Anytime Application
pub struct Standalone<'a> {
    /// url used for interface, modules, and the like
    url: String, 
    /// default json
    json: String,
    /// input/output midi
    midi: Midi,
    send_from_midi: cb::Sender<MidiMessage>,
    receive_from_midi: cb::Receiver<MidiMessage>,
    /// currently selected audio input device
    input_device: pa::DeviceIndex,
    /// currenlty selected audio outut device
    output_device: pa::DeviceIndex,
    /// GUI, only one instance for application, modules are injected iframe
    gui: GUI<'a>,
    /// incomming messages from GUI
    receive_from_gui: cb::Receiver<Message>,
    /// send from audio
    send_from_audio: Sender<(u32, Value)>,
    /// send from gui
    send_from_gui: cb::Sender<Message>,
    /// gui comms, for sending messages to GUI
    comms: Box<dyn Send>,
    comms_sender: cb::Sender<Message>,
}

impl <'a>Standalone<'a> {
    pub fn new(url: &str) -> Result<Self> {
       
        let mut midi = Midi::new();
        let midi_inputs = midi.get_inputs();
        println!("{:?}", midi_inputs);
        
        // Load GUI HTML, index.html is the same for all anywhere modules
        let html = get_string(&[url, "index.html"].join("/")).unwrap(); 
        let html = &[url, "index.html"].join("/");

        let modules = get_string(&[url, "modules.json"].join("/")).unwrap();
        Modules::from_json(&modules).and_then(|modules| {
            // thread communication channels
            let (send_from_midi, receive_from_midi) = cb::unbounded();
            let (send_from_gui, receive_from_gui) = cb::unbounded();
            let (send_from_audio, receive_from_audio) = channel();

            // default module to be loaded on startup
            let json = &modules.default.clone();

            Self::create_aaunit(url, json, send_from_audio.clone()).and_then(|(aaunit, bundle)| {
                let html = html.replace("$gui_page$", &bundle.gui.url)
                    .replace("$width$", &bundle.gui.width.to_string())
                    .replace("$height$", &bundle.gui.height.to_string());

                GUI::new(
                    &html[..],
                    Box::new(LocalSendCB::new(send_from_gui.clone())),
                    bundle.gui.params.clone(), //vec![Value::VFloat(-50.)],
                    "Audio Anywhere",
                    (900,900)).and_then(|gui| {
                        let pa = pa::PortAudio::new().unwrap();
                        let input_device = pa.default_input_device().unwrap();
                        let output_device = pa.default_output_device().unwrap();

                        let comms_sender = gui.comms_sender();
                        let comms = gui.comms();

                        // TODO: fix up unwrap()
                        // TODO: Add midi devices to GUI and allows selection
                        // midi.open_input(
                        //     "MPK Mini Mk II".to_string(), 
                        //     send_from_midi.clone(),
                        //     comms_sender.clone()).unwrap();
                        midi.open_input(
                            "MidiKeys".to_string(), 
                            send_from_midi.clone(),
                            comms_sender.clone()).unwrap();
                        // midi.open_input("MidiKeys".to_string(), send_from_midi.clone()).unwrap();
                        
                        // send Modules to GUI
                        Self::send_modules(&comms_sender, &modules.modules);
                        // send Audio devices to GUI
                        Self::send_audio_devices(&comms_sender);
                        // set default values for GUI and AAUnit
                        Self::send_params(&comms_sender, &bundle.gui.params);
                        Self::set_params(&aaunit, &bundle.gui.params);
                        
                        Ok(Self {
                            url: url.to_string(),
                            json: json.to_string(),
                            midi,
                            send_from_midi,
                            receive_from_midi,
                            input_device,
                            output_device,
                            gui,
                            receive_from_gui,
                            send_from_audio,
                            send_from_gui,
                            comms,
                            comms_sender,
                        })
                })
            })
        })
    }

    // send a list of input/output audio devices to GUI
    fn send_audio_devices(comms: &cb::Sender<Message>) {
        let pa = pa::PortAudio::new().unwrap();
        for device in pa.devices().unwrap() {
            let (index, info) = device.unwrap();
        
            if info.max_input_channels > 0 {
                Self::send_add_input_device(&comms, info.name, index);
            }

            if info.max_output_channels > 0 {
                Self::send_add_output_device(&comms, info.name, index);
            }
        }
    }

    // send a list of params settings, indexed by position in the vector, to GUI
    fn send_params(comms: &cb::Sender<Message>, params: &Vec<Value>) {
        for (index, p) in params.iter().enumerate() {
            comms.send(Message {
                id: MessageID::Param,
                index: index as Index, 
                value: (*p).clone(),
            }).unwrap();
        }
    }

    // send a list of modules to GUI
    fn send_modules(comms: &cb::Sender<Message>, modules: &Vec<Module>) {
        for m in modules {
            Self::send_add_module(comms, &m.name, &m.json_url);
        }
    }

    // send a message to GUI to add a module to drop down menu
    fn send_add_module(comms: &cb::Sender<Message>, name: &str, json_url: &str) {
        comms.send(Message {
            id: MessageID::AddModule,
            index: 0,
            value: Value::VString([name, json_url].join("="))
        }).unwrap();
    }

    // send a message to GUI to add an input audio device
    fn send_add_input_device(comms: &cb::Sender<Message>, name: &str, index: pa::DeviceIndex) {
        comms.send(Message {
            id: MessageID::AddInputDevice,
            index: 0,
            value: Value::VString([name, &index.0.to_string()].join("="))
        }).unwrap();
    }

    // send a message to GUI to add an output audio device
    fn send_add_output_device(comms: &cb::Sender<Message>, name: &str, index: pa::DeviceIndex) {
        comms.send(Message {
            id: MessageID::AddOutputDevice,
            index: 0,
            value: Value::VString([name, &index.0.to_string()].join("="))
        }).unwrap();
    }

    /// create an instance of an aaunit
    fn create_aaunit(url: &str, json: &str, send_from_audio: Sender<(u32, Value)>) -> Result<(AAUnit, Bundle)> {
        // firstly load the json bundle
        get_string(&[url, json].join("/")).and_then(|json| {
            Bundle::from_json(&json).and_then(|bundle| {
                // Load WASM module
                get_vec(&[url, &bundle.wasm_url].join("")).and_then(|wasm_code| {
                    if let Ok(aaunit) = AAUnit::new(&wasm_code[..]) {
                        Ok((aaunit, bundle))
                    }
                    else {
                        Err(())
                    }
                })
            })
        })
    }

    // set a aaunit parameter
    #[inline]
    fn set_param(aaunit: &AAUnit, index: Index, param: Value) {
        match param {
            Value::VFloat(f) => {
                let _ = aaunit.set_param_float(index, f);
            },
            Value::VInt(i) => {
                let _ = aaunit.set_param_int(index, i);
            },
            _ => {
            }
        }
    }

    // set aaunit parameters from a list of parameters
    fn set_params(aaunit: &AAUnit, params: &Vec<Value>) {
        for (index, param) in params.iter().enumerate() {
            Self::set_param(aaunit, index as u32, (*param).clone());
        }
    }

    /// audio handler for duplex streams (i.e. input and output)
    /// 0 < number inputs < 3 and 0 < number of outputs < 3
    fn audio_x_y(
        aaunit: Rc<RefCell<AAUnit>>, 
        input_device: pa::DeviceIndex,
        output_device: pa::DeviceIndex,
        bundle: Bundle, 
        receive_from_gui: cb::Receiver<Message>, 
        receive_from_midi: cb::Receiver<MidiMessage>,
        send_from_audio: cb::Sender<Message>) -> Option<Message> {
        let pa = pa::PortAudio::new().unwrap();

        let num_inputs = bundle.info.inputs;
        let num_outputs = bundle.info.outputs;

        let input_params = pa::stream::Parameters::new(
            input_device, 
            num_inputs,
            true,
            0.1);

        let output_params = pa::stream::Parameters::new(
            output_device, 
            num_outputs,
            true,
            0.1);

        let settings = 
            pa::stream::DuplexSettings::new(
                input_params, output_params, 44_100.0, 64);

        let (send_stop,rec_stop) = channel();
        let callback = move |pa::DuplexStreamCallbackArgs {
            in_buffer, 
            out_buffer, 
            frames, 
            .. }| { 
                // handle any incomming messages from MIDI
                loop {
                    if let Ok(message) = receive_from_midi.try_recv() {
                        println!("{:?}", message);
                    }
                    else {
                        break;
                    }
                }

                // handle any incomming messages from UI
                loop {
                    if let Ok(message) = receive_from_gui.try_recv() {
                        match message.id {
                            MessageID::Param => {
                                Self::set_param(&aaunit.borrow(), message.index, message.value);
                            },
                            MessageID::Control => {},
                            MessageID::ChangeModule 
                                | MessageID::AddInputDevice 
                                | MessageID::AddOutputDevice 
                                | MessageID::Exit => {
                                send_stop.send(Some(message.clone())).unwrap();
                                return pa::Complete;
                            },
                            _ => { }
                        }
                    }
                    else {
                        break;
                    }
                }                                                

                if num_inputs == 1 {
                    if num_outputs == 1 {
                        let _ = aaunit.borrow().compute_one_one(
                            frames, 
                            &in_buffer[..], 
                            &mut out_buffer[..]);
                    }
                    else {
                        let _ = aaunit.borrow().compute_one_two(
                            frames, 
                            &in_buffer[..], 
                            &mut out_buffer[..]);
                    }
                }
                else  {
                    if num_outputs == 1 {
                        let _ = aaunit.borrow().compute_two_one(
                            frames, 
                            &in_buffer[..], 
                            &mut out_buffer[..]);
                    }
                    else {
                        let _ = aaunit.borrow().compute_two_two(
                            frames, 
                            &in_buffer[..], 
                            &mut out_buffer[..]);
                    }
                }
                
                pa::Continue
        };

        let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();
        stream.start().unwrap();

        // block until we recieve message to swap module
        match rec_stop.recv() {
            Ok(s) => {
                stream.stop().unwrap();
                s 
            }
            _ => {
                stream.stop().unwrap();
                None
            }
        }
    }

    /// audio handler for output stream only
    fn audio_zero_x(
        aaunit: Rc<RefCell<AAUnit>>, 
        output_device: pa::DeviceIndex,
        bundle: Bundle, 
        receive_from_gui: cb::Receiver<Message>, 
        receive_from_midi: cb::Receiver<MidiMessage>,
        send_from_audio: cb::Sender<Message>) -> Option<Message> {
        let pa = pa::PortAudio::new().unwrap();

        let num_outputs = bundle.info.outputs;

        let output_params = pa::stream::Parameters::new(
            output_device, 
            num_outputs,
            true,
            0.1);

        let settings = 
            pa::stream::OutputSettings::new(output_params, 44_100.0, 64);        

        let (send_stop,rec_stop) = channel();
        let callback = move |pa::OutputStreamCallbackArgs {
            buffer, 
            frames, 
            .. }| { 
                // handle any incomming messages from MIDI
                loop {
                    if let Ok(message) = receive_from_midi.try_recv() {
                        match message.status() {
                            Status::NoteOn => {
                                let note     = message.data(1) as i32;
                                let velocity = message.data(2) as f32 / 127.0;
                                let _ = aaunit.borrow().handle_note_on(note, velocity);
                            },
                            Status::NoteOff => {
                                let note     = message.data(1) as i32;
                                let velocity = message.data(2) as f32 / 127.0;
                                let _ = aaunit.borrow().handle_note_off(note, velocity);
                            },
                            // Status::ControlChange => {
                            //     let controller = message.data(1);
                            //     let data       = message.data(2);
                            //     //send_stop.send(Some(message.clone())).unwrap();
                            //     //println!("{},{}", controller, data);
                            //     send_from_audio.send(Message {
                            //         id: MessageID::Control,
                            //         index: controller as Index,
                            //         value: Value::VInt(data as i32),
                            //     }).unwrap();
                            //     //println!("{:?}", message);
                            // },
                            _ => {},
                        }
                    }
                    else {
                        break;
                    }
                }

                // handle any incomming messages from UI
                loop {
                    if let Ok(message) = receive_from_gui.try_recv() {
                        match message.id {
                            MessageID::Param => {
                                Self::set_param(&aaunit.borrow(), message.index, message.value);
                            },
                            MessageID::Control => {},
                            MessageID::ChangeModule 
                                | MessageID::AddInputDevice 
                                | MessageID::AddOutputDevice 
                                | MessageID::Exit => {
                                send_stop.send(Some(message.clone())).unwrap();
                                return pa::Complete;
                            },
                            _ => { }
                        }
                    }
                    else {
                        break;
                    }
                }                                                

                if num_outputs == 1 {
                    let _ = aaunit.borrow().compute_zero_one(
                        frames,  
                        &mut buffer[..]);
                }
                else {
                    let _ = aaunit.borrow().compute_zero_two(
                        frames,  
                        &mut buffer[..]);
                }

                pa::Continue
        };

        let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();
        stream.start().unwrap();

        // block until we recieve message to swap module
        match rec_stop.recv() {
            Ok(s) => {
                stream.stop().unwrap();
                s 
            }
            _ => {
                stream.stop().unwrap();
                None
            }
        }
    }

    /// audio handler for  0:1, 1:1, 0:2, 1:2, 2:2 audio input:outputs
    /// currently limited to a maximum of stereo in out.
    #[inline]
    fn audio(
        aaunit: Rc<RefCell<AAUnit>>, 
        input_device: pa::DeviceIndex,
        output_device: pa::DeviceIndex,
        bundle: Bundle, 
        receive_from_gui: cb::Receiver<Message>, 
        receive_from_midi: cb::Receiver<MidiMessage>, 
        send_from_audio: cb::Sender<Message>) -> Option<Message> {

        // initialize the audio module
        let _ = aaunit.borrow_mut().init(44_100.0);

        // handle duplex or output only audio
        if bundle.info.inputs > 0 && bundle.info.outputs > 0 {
            Self::audio_x_y(
                aaunit, input_device, output_device, bundle, receive_from_gui, receive_from_midi, send_from_audio)
        }
        else if bundle.info.outputs > 0 {
            Self::audio_zero_x(aaunit, output_device, bundle, receive_from_gui, receive_from_midi, send_from_audio)
        }
        else {
            // TODO: add error! logging
            None
        }
    }

    /// Take hold of module a run Audio handler and GUI.
    /// The audio handler can be dynanically swapped on module change or input/output audio device change
    pub fn run(self) -> Result<()> {
        let mut gui = self.gui;
        let mut input_device = self.input_device;
        let mut output_device = self.output_device;
        let url = self.url;
        let receive_from_gui = self.receive_from_gui;
        let send_from_audio = self.send_from_audio;
        let comms = self.comms_sender;
        let json = self.json;
        let receive_from_midi = self.receive_from_midi;

        // create thread to handle all things audio...
        let audio_thread = thread::spawn(move || { 
            // we have to do this here, to avoid having to handle issues with wasmtime 
            // being initalized on the wrong thread.
            let (aaunit, bundle) = Self::create_aaunit(
                &url, 
                &json, 
                send_from_audio.clone()).unwrap();
            let aaunit = Rc::new(RefCell::new(aaunit));
            let mut bundle = bundle.clone();

            // audio can quit for a number of reasons:
            //          request change input/ouput device
            //          change audio anywhere module
            //          exit application
            //          unknown error
            while let Some(message) = Self::audio(
                            aaunit.clone(),
                            input_device,
                            output_device,
                            bundle.clone(), 
                            receive_from_gui.clone(),
                            receive_from_midi.clone(),
                            comms.clone()) {
                match message.id {
                    // switch input device
                    MessageID::AddInputDevice => {
                        if let Value::VInt(index) =  message.value {
                            input_device = pa::DeviceIndex(index as u32);
                        }
                    },
                    // switch output device
                    MessageID::AddOutputDevice => {
                        if let Value::VInt(index) =  message.value {
                            output_device = pa::DeviceIndex(index as u32);
                        }
                    },
                    // switch module
                    MessageID::ChangeModule => {
                        if let Value::VString(json) = message.value {
                            if let Ok((au, bundle_new)) = 
                                Self::create_aaunit(&url, &json, send_from_audio.clone()) {
                                comms.send(
                                    Message::change_module(
                                        &([&url[..], 
                                            &bundle_new.gui.url[..]].join("")), 
                                            bundle_new.gui.width, 
                                            bundle_new.gui.height)).unwrap();
                                
                                // set default values for GUI
                                Self::send_params(&comms, &bundle_new.gui.params);
                                Self::set_params(&au, &bundle_new.gui.params);

                                // finally install the auunit and bundle
                                *aaunit.borrow_mut() = au;
                                bundle = bundle_new;
                            }
                        }
                    },
                    MessageID::Exit => {
                        break;
                    },
                    _ => { }
                }
            }
        });

        gui.run();

        // clear up audio thread
        self.send_from_gui.send(Message {
            id: MessageID::Exit,
            index: 0,
            value: Value::VInt(0),
        }).unwrap();
        audio_thread.join().unwrap();
        
        Ok(())
    }
}