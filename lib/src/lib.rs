//! This crate acts as an interface that tetris bots can use to interact with a UI, but also to
//! generate moves and board states etc.

use std::io::{stdin, BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

mod tbp;
mod tetris;

pub use crate::tbp::*;
pub use crate::tetris::*;

pub trait Bot {
    fn new(board: Board) -> Self;
    fn search(&self, search_states: &SearchStatus);
}

pub struct SearchStatus {
    terminate: Arc<AtomicBool>,
    want_moves: Arc<AtomicBool>,
    new_piece: Arc<Mutex<Option<Piece>>>,
    suggestion_sender: Sender<Vec<Move>>,
    // Receive a move when the Play tbp command is sent
    play_move: Arc<Mutex<Option<Move>>>,
}

impl SearchStatus {
    pub fn terminate(&self) -> bool {
        !self.terminate.load(Ordering::Acquire)
    }
    pub fn current_moves(&self, moves: &Vec<Move>) {
        if self.want_moves.load(Ordering::Acquire) {
            self.suggestion_sender.send(moves.to_vec()).unwrap();
            self.want_moves.store(false, Ordering::Release);
        }
    }
    pub fn new_move(&self) -> Option<Move> {
        let mut move_ptr = self.play_move.lock().unwrap();
        if let Some(mv) = *move_ptr {
            *move_ptr = None;
            return Some(mv);
        }
        None
    }
    pub fn new_piece(&self) -> Option<Piece> {
        let mut piece_ptr = self.new_piece.lock().unwrap();
        if let Some(piece) = *piece_ptr {
            *piece_ptr = None;
            return Some(piece);
        }
        None
    }
}

pub fn run_bot<B: Bot>(info: BotInfo) {
    BotMessage::Info(info).send_message();

    let calculating = Arc::new(AtomicBool::new(false));
    let want_moves = Arc::new(AtomicBool::new(false));
    let play_move: Arc<Mutex<Option<Move>>> = Arc::new(Mutex::new(None));
    let new_piece: Arc<Mutex<Option<Piece>>> = Arc::new(Mutex::new(None));

    let mut reader = BufReader::new(stdin()).lines();
    let (move_sender, move_receiver) = channel();

    while let Some(Ok(line)) = reader.next() {
        let msg = get_frontend_message(line).unwrap();
        match msg {
            FrontendMessage::Rules => {
                // Maybe one day the rules of a game will be parsed here, but right now this
                // message is empty
                BotMessage::Ready.send_message();
            }
            FrontendMessage::Quit => {
                break;
            }
            FrontendMessage::Stop => {
                // Tell the bot to stop calculating
                calculating.store(false, Ordering::Release);
            }
            FrontendMessage::Suggest => {
                // Tell the bot to suggest some moves in order of preference.
                // Only valid to recieve if bot is calculating
                // Bot should send a suggestion message
                want_moves.store(true, Ordering::Release);
                BotMessage::Suggestion {
                    moves: move_receiver.recv().unwrap(),
                }
                .send_message();
            }
            FrontendMessage::Play { mv } => {
                // Tell the bot to update the game state by applying the move specified and begin
                // calculating from the new position
                // Only valid to recieve if bot is calculating
                // Hold is inferred by the move piece
                let mut move_ptr = play_move.lock().unwrap();
                *move_ptr = Some(mv);
            }
            FrontendMessage::NewPiece { piece } => {
                // Tell the bot that a new piece is added to the queue
                let mut piece_ptr = new_piece.lock().unwrap();
                *piece_ptr = Some(piece);
            }
            FrontendMessage::Start(tbp_board) => {
                // Tell the bot to begin calculating the given position
                if calculating.load(Ordering::Acquire) {
                    println!("Bot was already calculating");
                } else {
                    // Create the board based on input
                    let board = Board::from_tbp(tbp_board);
                    calculating.store(true, Ordering::Release);
                    thread::spawn({
                        let terminate = calculating.clone();
                        let want_moves = want_moves.clone();
                        let new_piece = new_piece.clone();
                        let move_sender = move_sender.clone();
                        let play_move = play_move.clone();
                        move || {
                            let search_status = SearchStatus {
                                terminate,
                                want_moves,
                                new_piece,
                                suggestion_sender: move_sender,
                                play_move,
                            };
                            let bot = B::new(board);
                            bot.search(&search_status);
                        }
                    });
                }
            }
        }
    }
}
