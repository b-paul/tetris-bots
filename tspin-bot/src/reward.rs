use lib::tetris::BbBoard;
use crate::q_function::QFunction;

pub trait Reward {
    fn reward(&self) -> f64;
    fn shaped_reward(&self) -> f64;
    fn search_for_reward<Q: QFunction>(&self, learning_rate: f64,  q_function: &Q) -> f64;
}

impl Reward for BbBoard {
    fn reward(&self) -> f64 {
        // death check
        if !self.queue.is_empty() {
            let moves = self.gen_moves();
            if moves.is_empty() {
                return -999999.;
            }
        }

        if self.wasted_ts != 0 {
            return -999999.;
        }

        // Also add a thing for -999999 score if you have done any line clears that arent tspins

        (self.tspins * 100) as f64
    }

    fn shaped_reward(&self) -> f64 {
        if self.wasted_ts != 0 {
            -999999.
        } else {
            (self.tspins * 100) as f64
        }
    }

    fn search_for_reward<Q: QFunction>(&self, learning_rate: f64,  q_function: &Q) -> f64 {
        let moves = self.gen_moves();
        if moves.is_empty() {
            self.reward()
        } else {
            self.shaped_reward() + learning_rate * q_function.best_score(self).unwrap_or(0.)
        }
    }
}
