use crate::tbp::BotInfo;
use crate::tetris::*;
use lib::*;

pub mod mab;
pub mod mcts;
pub mod nn;
pub mod q_function;
pub mod reward;
pub mod simulate;

struct TspinBot {
    board: BbBoard,
}

impl Bot for TspinBot {
    type BoardType = BbBoard;
    type MoveType = BbMove;

    fn new(board: BbBoard) -> Self {
        TspinBot { board }
    }

    fn search(&self, search_status: &SearchStatus<BbMove>) {
        let q_function = q_function::Baby::new();
        /*
        let mut q_function = nn::Network::new_random();
        // now train this! but first we need a list of positions to train with... :(
        // Make 100000 boards
        let mut training_boards = vec![];
        for _ in 0..1 {
            let mut board = self.board.clone();
            board.make_queue_len(20);
            while !board.queue.is_empty() {
                let mv = mcts::internal_mcts(&mut board, &q_function);
                board.make_move_in_place(mv);
            }
            board.make_queue_len(100);
            training_boards.push(board);
        }
        println!("Generated training boards");
        let mut strategy = mab::UCB1::new();
        q_function.train(1, &training_boards, &mut strategy);

        println!("{:?}", q_function);
        */

        let new_board = &mut self.board.clone();

        let mut mv = self.get_best_move(search_status, new_board, &q_function);

        loop {
            search_status.current_moves(&[mv]);
            if search_status.terminate() {
                break;
            }
            if let Some(new_move) = search_status.new_move() {
                new_board.make_move_in_place(new_move);
                mv = self.get_best_move(search_status, new_board, &q_function);
            }
            if let Some(piece) = search_status.new_piece() {
                new_board.queue.push(piece);
            }
        }
    }
}

impl TspinBot {
    // Monte carlo tree search which has a policy network and a value network
    // Value network is trained on games from when the policy network is being trained
    // Train the policy network with games with fixed 100(or something) piece queues, but only give
    // the policy network the normal queue etc
    fn get_best_move<Q: q_function::QFunction>(&self, status: &SearchStatus<BbMove>, board: &mut BbBoard, q_function: &Q) -> BbMove {
        mcts::mcts(status, board, q_function)
    }
}

fn main() {
    run_bot::<TspinBot>(BotInfo {
        name: "Tspinning bot",
        author: "bpaul",
        version: "0",
        features: &[],
    });
}
