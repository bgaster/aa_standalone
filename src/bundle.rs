use serde::{Deserialize};

use crate::utils::{err, ok, Result};
use crate::messages::*;

#[derive(Deserialize, Debug, Clone)]
pub struct GUIBundle {
    pub url: String,
    pub name: String,
    pub params: Vec<Value>,
    pub width: i32,
    pub height: i32, 
}

#[derive(Deserialize, Debug, Clone)]
pub struct Info {
    pub name: String,
    pub vendor: String,
    pub presets: u32,
    pub parameters: u32,
    pub inputs: i32,
    pub outputs: i32,
    pub midi_inputs: u32,
    pub midi_outputs: u32,
    pub id: u32,
    pub version: u32,
    pub category: String,
    pub initial_delay: u32,
    pub preset_chunks: bool,
    pub f64_precision: bool,
    pub silent_when_stopped: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Bundle {
    pub wasm_url: String,
    pub gui: GUIBundle,
    pub info: Info,
}

impl Bundle {
    pub fn from_json(data: &str) -> Result<Self> {
        let bundle : serde_json::Result<Bundle> = serde_json::from_str(data);
        //println!("{:?}", bundle);
        bundle.map_or(err(), |b| ok(b))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Module {
    pub name: String,
    pub json_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Modules {
    pub default: String,
    pub modules: Vec<Module>,
}

impl Modules {
    pub fn from_json(data: &str) -> Result<Self> {
        let modules : serde_json::Result<Modules> = serde_json::from_str(data);
        modules.map_or(err(), |b| ok(b))
    }
}