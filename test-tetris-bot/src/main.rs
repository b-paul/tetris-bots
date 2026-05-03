use crate::tbp::BotInfo;
use crate::tetris::*;
use lib::*;

struct BadBot {
    board: BbBoard,
}

impl Bot for BadBot {
    type BoardType = BbBoard;
    type MoveType = BbMove;

    fn new(board: BbBoard) -> Self {
        BadBot { board }
    }

    fn search(&self, search_status: &SearchStatus<BbMove>) {
        let new_board = &mut self.board.clone();

        let mut mv = self.get_best_move(new_board);

        loop {
            search_status.current_moves(&[mv]);
            if search_status.terminate() {
                break;
            }
            if let Some(new_move) = search_status.new_move() {
                new_board.make_move_in_place(new_move);
                mv = self.get_best_move(new_board);
            }
            if let Some(piece) = search_status.new_piece() {
                new_board.queue.push(piece);
            }
        }
    }
}

fn evaluate(board: &BbBoard) -> isize {
    let mut holes = 0;
    let mut max_y = 0;
    for x in 0..10 {
        let top_y = 64 - board.board[x].leading_zeros() as isize;
        if top_y > max_y {
            max_y = top_y;
        }
        for y in (0..40).rev() {
            if board.board[x] & (1 << y) != 0 {
                for i in 0..y {
                    if board.board[x] & (1 << i) == 0 {
                        holes += 1;
                    }
                }
                break;
            }
        }
    }

    let mut score =
        max_y * 20 + 10 * holes - 50 * board.tspins as isize + 100 * board.wasted_ts as isize;
    if board.back_to_back {
        score -= 50;
    }
    if max_y > 10 {
        score += 800 + 15 * holes
    }

    score
}

fn search(board: &BbBoard, depth: u8) -> isize {
    if depth == 0 || board.queue.is_empty() {
        return evaluate(board);
    }
    let mut moves: Vec<(BbMove, isize)> = board
        .gen_moves()
        .iter()
        .map(|mv| (*mv, evaluate(&board.make_move(*mv))))
        .collect();
    moves.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

    let mut best_score = 999999;

    let mut count = 0;

    for (mv, _) in moves {
        count += 1;
        if count > 7 {
            //break;
        }
        let score = search(&board.make_move(mv), depth - 1);
        if score < best_score {
            best_score = score;
        }
    }

    best_score
}

impl BadBot {
    fn get_best_move(&self, board: &BbBoard) -> BbMove {
        let mut moves: Vec<(BbMove, isize)> = board
            .gen_moves()
            .iter()
            .map(|mv| (*mv, evaluate(&board.make_move(*mv))))
            .collect();
        moves.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

        let mut best_move = moves[0].0;
        let mut best_score = 999999;

        let mut count = 0;

        for (mv, _) in moves {
            count += 1;
            if count > 7 {
                //break;
            }
            let score = search(&board.make_move(mv), 2);
            if score < best_score {
                best_move = mv;
                best_score = score;
            }
        }

        best_move
    }
}

fn main() {
    run_bot::<BadBot>(BotInfo {
        name: "Test bot",
        author: "bpaul",
        version: "0",
        features: &[],
    });
}
