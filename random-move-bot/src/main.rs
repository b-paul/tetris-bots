use lib::*;
use rand::Rng;

struct RandomBot {
    board: Board,
}

impl Bot for RandomBot {
    fn new(board: Board) -> Self {
        RandomBot { board }
    }
    fn search(&self, search_status: &SearchStatus) {
        let mutable_board = &mut self.board.clone();

        let mut mv = self.get_move(mutable_board);

        loop {
            search_status.current_moves(&vec![mv]);
            if search_status.terminate() {
                break;
            }
            if let Some(new_move) = search_status.new_move() {
                mutable_board.make_move_in_place(new_move);
                mv = self.get_move(mutable_board);
            }
            if let Some(piece) = search_status.new_piece() {
                mutable_board.queue.push(piece);
            }
        }
    }
}

impl RandomBot {
    fn get_move(&self, board: &Board) -> Move {
        let moves = board.gen_moves();
        moves[rand::thread_rng().gen_range(0..moves.len())]
    }
}

fn main() {
    run_bot::<RandomBot>(BotInfo {
        name: "Random Move Bot",
        author: "bpaul",
        version: "v1 (the only version)",
        features: &[],
    });
}
