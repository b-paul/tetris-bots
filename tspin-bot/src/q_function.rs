use lib::tetris::{BbBoard, BbMove};

pub trait QFunction {
    fn call(&self, board: &BbBoard, mv: &BbMove) -> f64;
    fn best_score(&self, board: &BbBoard) -> Option<f64> {
        let moves = board.gen_moves();
        let best_action = self.best_action(board, &moves)?;
        Some(self.call(board, &best_action))
    }
    fn best_action(&self, board: &BbBoard, moves: &[BbMove]) -> Option<BbMove> {
        if moves.is_empty() {
            return None;
        }
        let mut best_mv = moves[0];
        let mut best_q = -999999.;
        for mv in moves {
            let q = self.call(board, mv);
            if q > best_q {
                best_q = q;
                best_mv = *mv;
            }
        }
        Some(best_mv)
    }
}

// lmoa please make an actual q function implementation
pub struct Baby {}

impl Default for Baby {
    fn default() -> Self {
        Self::new()
    }
}

impl Baby {
    pub fn new() -> Baby {
        Baby {}
    }
}

impl QFunction for Baby {
    fn call(&self, board: &BbBoard, _mv: &BbMove) -> f64 {
        let mut holes = 0;
        let mut max_y = 0;
        for x in 0..10 {
            let top_y = 63 - board.board[x].leading_zeros() as isize;
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

        let mut score = max_y * 40 + 10 * holes;
        if max_y > 10 {
            score += 800 + 15 * holes
        }

        //-score as f64
        if board.wasted_ts != 0 {
            -999999.
        } else {
            (board.tspins * 100) as f64 - (score as f64)
        }
    }
}
