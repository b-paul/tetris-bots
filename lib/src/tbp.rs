extern crate serde;

use serde::{Deserialize, Serialize};

pub use crate::tetris::*;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FrontendMessage {
    Rules,
    Start(TBPBoard),
    Stop,
    Suggest,
    Play {
        #[serde(rename = "move")]
        mv: Move,
    },
    NewPiece {
        piece: Piece,
    },
    Quit,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BotMessage {
    Error { reason: String },
    Ready,
    Info(BotInfo),
    Suggestion { moves: Vec<Move> },
}

#[derive(Deserialize)]
pub struct TBPBoard {
    pub hold: Option<Piece>,
    pub queue: Vec<Piece>,
    pub combo: u32,
    pub back_to_back: bool,
    pub board: Vec<Vec<Option<char>>>,
}

#[derive(Serialize)]
pub struct BotInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub features: &'static [&'static str],
}

impl BotMessage {
    pub fn send_message(&self) {
        let str = serde_json::to_string(self).unwrap();
        if str == "Null" {
            panic!("null thing!?!?");
        }
        println!("{}", str);
    }
}

pub fn get_frontend_message(input: String) -> serde_json::Result<FrontendMessage> {
    let v: FrontendMessage = serde_json::from_str(&input)?;

    Ok(v)
}
