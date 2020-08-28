use serde::{Deserialize};
use serde_repr::{Deserialize_repr};

extern crate crossbeam_channel;
use crossbeam_channel as cb;
//use crossbeam_queue::{ArrayQueue, PushError};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;

use web_view::*;

use crate::messages::*;
use crate::comms::*;

#[derive(Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u16)]
#[serde(untagged)]
pub enum MsgType {
    Console = 0,
    SendParam = 1,
    ChangeModule = 2,
    ChangeInputDevice = 3,
    ChangeOutputDevice = 4,
    Loaded = 5,
    NoteOn = 6,
    NoteOff = 7,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UIMessage  {
    pub msg: MsgType,
    pub index: u32,
    pub value: Option<Value>,
}

type JavascriptCallback = Box<dyn FnMut(&mut web_view::WebView<()>, &str) -> WVResult>;

struct Handler {
    sender: Box<dyn Send>,
    gui_sender: cb::Sender<Message>,
}

impl Handler  {
    pub fn new(sender: Box<dyn Send>, gui_sender: cb::Sender<Message>) -> Self {
        Self {
            sender,
            gui_sender,
        }
    }

    pub fn receive(&mut self, index: Index, value: Value) {
        self.sender.send(MessageID::Param, index, value ).unwrap();
    }

    pub fn console(&self, s: &str) -> () {
        println!("{}", s);
    }

    pub fn change_module(&mut self, value: Value) {
        self.sender.send(MessageID::ChangeModule, 0, value).unwrap();
    }

    pub fn add_input_device(&mut self, value: Value) {
        self.sender.send(MessageID::AddInputDevice, 0, value).unwrap();
    }

    pub fn add_output_device(&mut self, value: Value) {
        self.sender.send(MessageID::AddOutputDevice, 0, value).unwrap();
    }

    pub fn note_on(&mut self, value: Value) {
        self.sender.send(MessageID::NoteOn, 0, value);
    }

    pub fn note_off(&mut self, value: Value) {
        self.sender.send(MessageID::NoteOff, 0, value);
    }

    pub fn loaded(&mut self) {
        self.gui_sender.send(Message {
            id: MessageID::Loaded,
            index: 0,
            value: Value::VInt(0),
        }).unwrap();
    }
}

pub struct GUI<'a> {
    webview: WebView<'a, ()>,
    // true once webview has loaded, message sent from onLoad
    loaded: bool,
    size: (i32,i32),
    is_open: bool,
    external_sender: cb::Sender<Message>,
    external_receiver: cb::Receiver<Message>,
    //queue: ArrayQueue<Message>,
}

impl <'a> GUI<'a> {
    pub fn new(
        html: &str, 
        audio_sender: Box<dyn Send>,
        params: Vec<Value>,
        title: &'a str,
        size: (i32,i32)) -> Result<Self, ()> {

        let (external_sender, external_receiver) = cb::unbounded();

        //let queue = ArrayQueue::new(1024);

        let handler = Handler::new(audio_sender, external_sender.clone());

        match web_view::builder()
            .title(title)
            .content(Content::Url(html))
            .size(size.0, size.1)
            .resizable(false)
            .debug(true)
            .user_data(())
            .invoke_handler(Self::create_javascript_callback(handler))
            .build() {
                Ok(webview) => {
                    Ok(GUI {
                        webview,
                        loaded: false,
                        size,
                        is_open: false,
                        external_sender,
                        external_receiver
                    })
                },
                _ => {
                    Err(())
                }
            }
    }

    pub fn comms_sender(&self) -> cb::Sender<Message> {
        self.external_sender.clone()
    }

    pub fn comms(&self) -> Box<dyn Send> {
        Box::new(LocalSend::new(self.external_sender.clone()))
    } 

    pub fn _comms_arc(&self) -> Arc<dyn Send> {
        Arc::new(LocalSend::new(self.external_sender.clone()))
    }

    pub fn close(&mut self) {
        self.webview.exit();
    } 

    pub fn run(&mut self) {
        self.is_open = true;
        
        let mut msgs = Vec::new();

        loop {
            // process any incoming messages
            loop {
                match self.external_receiver.try_recv() {
                    Ok(m) => {
                        match m.id {
                            MessageID::Loaded => {
                                self.loaded = true;
                            },
                            _ => {
                                msgs.push(m.clone());
                            }
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }

            if self.loaded {
                let mut msgs_consumed = 0;
                for m in msgs.iter() {
                    match (*m).id {
                        MessageID::NoteOn => {

                        },
                        MessageID::NoteOff => {

                        },
                        MessageID::Param => {
                            Self::param_change(&mut self.webview, (*m).index, (*m).value.to_string()).unwrap();
                            msgs_consumed += 1;
                        }
                        MessageID::Control => {
                            //println!("{:?}", (*m).value.to_string());
                            Self::control_change(&mut self.webview, (*m).index, i32::from((*m).value.clone())).unwrap();
                            msgs_consumed += 1;
                        }
                        MessageID::ChangeModule => {
                            let str = (*m).value.to_string().clone();
                            let args: Vec<&str> = str.split_whitespace().collect();
                            self.loaded = false;
                            Self::change_module(&mut self.webview, args[0], args[1], args[2]).unwrap();
                            msgs_consumed += 1;
                            break;
                        },
                        MessageID::AddInputDevice => {
                            let str = (*m).value.to_string().clone();
                            let args: Vec<&str> = str.split("=").collect();
                            Self::add_input_device(&mut self.webview, args[0], args[1]).unwrap();
                            msgs_consumed += 1;
                        },
                        MessageID::AddOutputDevice => {
                            let str = (*m).value.to_string().clone();
                            let args: Vec<&str> = str.split("=").collect();
                            Self::add_output_device(&mut self.webview, args[0], args[1]).unwrap();
                            msgs_consumed += 1;
                        },
                        MessageID::Exit => {
                            // TODO: add Exit message?
                            msgs_consumed += 1;
                        },
                        MessageID::AddModule => {
                            let str = (*m).value.to_string().clone();
                            let args: Vec<&str> = str.split("=").collect();
                            Self::add_module(&mut self.webview, args[0], args[1]).unwrap();
                            msgs_consumed += 1;
                        },
                        _ => {
                            msgs_consumed += 1;
                            // no need to handle loaded
                        }
                    }
                }
                msgs.drain(0..msgs_consumed);
            }

            // step the webview
            match self.webview.step() {
                Some(Ok(_)) => (),
                Some(e) => e.unwrap(),
                None => {
                    break;
                },
            }
        }
    }

    fn create_javascript_callback(mut handler: Handler) -> JavascriptCallback {
        Box::new(move |webview: &mut web_view::WebView<()>, args: &str| {
            let message : serde_json::Result<UIMessage> = serde_json::from_str(args);
           //println!("{:?} {}", message, args);
            match message {
                Ok(message) => {
                    match message.msg {
                            MsgType::NoteOn => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.note_on(v); Ok(()) });
                            },
                            MsgType::NoteOff => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.note_off(v); Ok(()) });
                            },
                            MsgType::SendParam => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.receive(message.index, v); Ok(()) });
                            },
                            MsgType::Console => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.console(&v.to_string()[..]); Ok(()) });
                            },
                            MsgType::ChangeModule => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.change_module(v); Ok(()) });
                            },
                            MsgType::ChangeInputDevice => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.add_input_device(v); Ok(()) });
                            },
                            MsgType::ChangeOutputDevice => {
                                return message.value.clone()
                                    .map_or(Ok(()), |v| { handler.add_output_device(v); Ok(()) });
                            },
                            MsgType::Loaded => {
                                handler.loaded();
                            }
                        }
                },
                _ => {}
            }
            Ok(())
        })
    }

    fn param_change(webview: &mut WebView<()>, param_index: u32, data: String) -> WVResult {
        webview.eval(&format!("OnParamChange({},{})", param_index, data)).unwrap();
        Ok(())
    }

    fn control_change(webview: &mut WebView<()>, ctrl_index: u32, data: i32) -> WVResult {
        webview.eval(&format!("OnControlChange({},{})", ctrl_index, data)).unwrap();
        Ok(())
    }

    fn change_module(webview: &mut WebView<()>, url: &str, width: &str, height: &str) -> WVResult {
        let eval = format!("OnModuleChange(\"{}\",\"{}\",\"{}\")", url, width, height);
        webview.eval(&eval).unwrap();
        Ok(())
    }

    fn add_input_device(webview: &mut WebView<()>, name: &str, index: &str) -> WVResult {
        webview.eval(&format!("OnAddInputDevice({},\"{}\")", index, name)).unwrap();
        Ok(())
    }

    fn add_output_device(webview: &mut WebView<()>, name: &str, index: &str) -> WVResult {
        webview.eval(&format!("OnAddOutputDevice({},\"{}\")", index, name)).unwrap();
        Ok(())
    }

    fn add_module(webview: &mut WebView<()>, name: &str, json_url: &str) -> WVResult {
        webview.eval(&format!("OnAddModule(\"{}\",\"{}\")", name, json_url)).unwrap();
        Ok(())
    }
}