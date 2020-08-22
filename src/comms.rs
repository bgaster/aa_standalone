#![allow(dead_code)]

use std::sync::mpsc::{Sender};

use crate::messages::*;
use crossbeam_channel as cb;

type CommsError = ();

// many parts of the system run completly asyncronously and care is taken to avoid using blocking commnication between 
// any component. This has the advantage of not only avoid any blocking component, but also allow the GUI, for 
// example, to be run remotely. 
//
// In most cases there is expected to be only a single receiver, but in some cases this does not work and multiple 
// receivers are support, but must be requested explicitly.

/// types of messages that can be sent between components
/// some messages are bi-directional (e.g. ChangeModule is sent both ways between GUI and Audio),
/// while others are not (e.g. Loaded, is GUI specific)
#[derive(Clone, PartialEq, Debug)]
pub enum MessageID {
    /// parameter change
    Param = 0,
    /// control change
    Control = 1,
    /// change current module
    ChangeModule = 2,
    /// add input device (to GUI)
    AddInputDevice = 3,
    /// add output device (to GUI)
    AddOutputDevice = 4,
    /// GUI has completed loading
    Loaded = 5,
    /// Application quit
    Exit = 6,
    /// add module (to GUI)
    AddModule = 7,
}

/// Simple message format used to communicate between different components, in particular, 
/// the GUI and audio elements.
#[derive(Clone, Debug)]
pub struct Message {
    /// id (type) of meesage
    pub id: MessageID,
    /// index into data (often not used, in that case set to 0)
    pub index: Index,
    /// datagram of message
    pub value: Value,
}

impl Message {
    /// utility function to create a change module message
    #[inline]
    pub fn change_module(url: &str, width: i32, height: i32) -> Self {
        Message {
            id: MessageID::ChangeModule,
            index: 0,
            value: Value::VString([url, &width.to_string(), &height.to_string()].join(" ")),
        }
    }
}

pub trait Send {
    fn send(&self, id: MessageID, index: Index, v: Value) -> Result<(), CommsError>;
}

pub trait Receive {
    fn recv(&self, index: Index) -> Result<(MessageID, Value), CommsError>;
    fn try_recv(&self, index: Index) -> Result<(MessageID, Value), CommsError>;
}

//-----------------------------------------------------------------------------

pub struct LocalSend {
    sender: cb::Sender <Message>,
}

impl LocalSend {
    pub fn new(sender: cb::Sender <Message>) -> Self {
        Self {
            sender,
        }
    }
}

impl Send for LocalSend {
    fn send(&self, id: MessageID, index: Index, value: Value) -> Result<(), ()> {
        self.sender.send(Message { id, index, value }).map_or(Err(()), |_| Ok(()))
    }
}

pub struct LocalSendCB {
    sender: cb::Sender <Message>,
}

impl LocalSendCB {
    pub fn new(sender: cb::Sender <Message>) -> Self {
        Self {
            sender,
        }
    }
}

impl Send for LocalSendCB {
    fn send(&self, id: MessageID, index: Index, value: Value) -> Result<(), ()> {
        self.sender.send(Message { id, index, value }).map_or(Err(()), |_| Ok(()))
    }
}

//TODO:
//struct RemoteSend;

//-----------------------------------------------------------------------------
