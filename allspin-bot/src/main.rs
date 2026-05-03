use lib::tbp::BotInfo;
use lib::tetris::{BbBoard, BbMove};
use lib::{Bot, SearchStatus, run_bot};

// PROBLEMS:
// it sometimes makes a two wide well on the left/right side of the board, then fails to do anything
// with it while keeping b2b and dies
mod eval;

struct AllspinBot {
    board: BbBoard,
}

impl Bot for AllspinBot {
    type BoardType = BbBoard;

    type MoveType = BbMove;

    fn new(board: Self::BoardType) -> Self {
        AllspinBot { board }
    }

    fn search(&mut self, search_status: &SearchStatus<Self::MoveType>) {
        let mut mvs = self.get_best_moves();

        loop {
            search_status.current_moves(&[mvs[0]]);
            if search_status.terminate() {
                break;
            }
            if let Some(mut new_move) = search_status.new_move() {
                new_move.spin = lib::tbp::Spin::Mini;
                self.board.make_move_in_place(new_move);
                mvs = self.get_best_moves();
            }
            if let Some(piece) = search_status.new_piece() {
                self.board.queue.push(piece);
            }
        }
    }
}

#[derive(Clone)]
struct Node {
    root_mv: BbMove,
    board: BbBoard,
    score: i64,
}

const MAX_DEPTH: usize = 7;
const BEAM_WIDTH: usize = 1000;

impl AllspinBot {
    fn get_best_moves(&self) -> Vec<BbMove> {
        let mut beam: Vec<_> = self
            .board
            .gen_moves()
            .iter()
            .map(|&mv| {
                let board = self.board.make_move(mv);
                Node {
                    root_mv: mv,
                    score: eval::eval(&board),
                    board,
                }
            })
            .collect();

        beam.sort_by_key(|n| std::cmp::Reverse(n.score));
        if beam.len() > BEAM_WIDTH {
            beam.drain(BEAM_WIDTH..);
        }

        for _ in 0..MAX_DEPTH {
            let mut new_beam = Vec::new();

            for node in &beam {
                new_beam.extend(node.board.gen_moves().iter().map(|&mv| {
                    let board = node.board.make_move(mv);
                    Node {
                        root_mv: node.root_mv,
                        score: eval::eval(&board),
                        board,
                    }
                }));
            }
            if new_beam.is_empty() {
                break;
            }
            beam = new_beam;
            beam.sort_by_key(|n| std::cmp::Reverse(n.score));
            if beam.len() > BEAM_WIDTH {
                beam.drain(BEAM_WIDTH..);
            }
        }

        beam.into_iter().map(|n| n.root_mv).collect()
    }
}

fn main() {
    run_bot::<AllspinBot>(BotInfo {
        name: "All spin bot",
        author: "bpaul",
        version: "v0",
        features: &[],
    });
}
