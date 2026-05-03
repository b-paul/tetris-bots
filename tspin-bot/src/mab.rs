// Multi armed bandit strategies
use crate::q_function::QFunction;
use lib::tetris::{BbBoard, BbMove};
use rand::{prelude::SliceRandom, Rng};
use std::collections::HashMap;

pub trait Strategy {
    fn reset(&mut self);
    fn select<Q: QFunction>(
        &mut self,
        board: &BbBoard,
        moves: &[BbMove],
        q_function: &Q,
    ) -> Option<BbMove>;
}

pub struct EpsilonGreedy {
    epsilon: f64,
}

impl EpsilonGreedy {
    pub fn new(epsilon: f64) -> Self {
        EpsilonGreedy { epsilon }
    }
}

impl Strategy for EpsilonGreedy {
    fn reset(&mut self) {}

    fn select<Q: QFunction>(
        &mut self,
        board: &BbBoard,
        moves: &[BbMove],
        q_function: &Q,
    ) -> Option<BbMove> {
        let mut rng = rand::thread_rng();
        let rand: f64 = rng.gen();
        if rand < self.epsilon {
            // Pick the best action
            q_function.best_action(board, moves)
        } else {
            // Pick a random action
            // dont unwrap! use an option!
            moves.choose(&mut rng).copied()
        }
    }
}

pub struct EpsilonDecreasing {
    alpha: f64,
    epsilon: f64,
    start_epsilon: f64,
}

impl EpsilonDecreasing {
    pub fn new(alpha: f64, epsilon: f64) -> Self {
        EpsilonDecreasing {
            alpha,
            epsilon,
            start_epsilon: epsilon,
        }
    }
}

impl Strategy for EpsilonDecreasing {
    fn reset(&mut self) {
        self.epsilon = self.start_epsilon;
    }

    fn select<Q: QFunction>(
        &mut self,
        board: &BbBoard,
        moves: &[BbMove],
        q_function: &Q,
    ) -> Option<BbMove> {
        let mut rng = rand::thread_rng();
        let rand: f64 = rng.gen();
        let result = if rand < self.epsilon {
            // Pick the best action
            q_function.best_action(board, moves)
        } else {
            // Pick a random action
            // dont unwrap! use an option!
            moves.choose(&mut rng).copied()
        };
        self.epsilon *= self.alpha;
        result
    }
}

pub struct Softmax {
    temperature: f64,
}

impl Softmax {
    pub fn new(temperature: f64) -> Self {
        Softmax { temperature }
    }
}

impl Strategy for Softmax {
    fn reset(&mut self) {}
    fn select<Q: QFunction>(
        &mut self,
        board: &BbBoard,
        moves: &[BbMove],
        q_function: &Q,
    ) -> Option<BbMove> {
        let total: f64 = moves
            .iter()
            .map(|mv| (q_function.call(board, mv) / self.temperature).exp())
            .sum();

        let mut rng = rand::thread_rng();
        let rand: f64 = rng.gen();

        let mut cumulative_prob = 0.;

        for mv in moves {
            let term = (q_function.call(board, mv) / self.temperature).exp() / total;

            if cumulative_prob + term > rand {
                return Some(*mv);
            }

            cumulative_prob += term;
        }

        // This shouldn't be reached ever I hope
        None
    }
}

pub struct UCB1 {
    // the amount of times each action has been made
    hash_map: HashMap<BbMove, usize>,
    total: usize,
}

impl Default for UCB1 {
    fn default() -> Self {
        Self::new()
    }
}

impl UCB1 {
    pub fn new() -> Self {
        UCB1 {
            hash_map: HashMap::new(),
            total: 0,
        }
    }

    fn num_of_move(&self, mv: &BbMove) -> usize {
        self.hash_map.get(mv).copied().unwrap_or(0)
    }

    fn update_counter(&mut self, mv: &BbMove) {
        if let Some(count) = self.hash_map.get_mut(mv) {
            *count += 1;
        } else {
            self.hash_map.insert(*mv, 1);
        }
    }
}

impl Strategy for UCB1 {
    fn reset(&mut self) {
        self.hash_map.clear();
        self.total = 0
    }

    fn select<Q: QFunction>(
        &mut self,
        board: &BbBoard,
        moves: &[BbMove],
        q_function: &Q,
    ) -> Option<BbMove> {
        let mut best_move = moves[0];
        let mut best_val = -999999.;

        for mv in moves {
            let num_of_move = self.num_of_move(mv);
            if num_of_move == 0 {
                // if we havent searched this move before, search it
                self.update_counter(mv);
                return Some(*mv);
            }

            let val = q_function.call(board, mv)
                - (((2. * self.total as f64).ln()) / (num_of_move as f64)).sqrt();

            if val > best_val {
                best_move = *mv;
                best_val = val;
            }
        }

        self.update_counter(&best_move);
        Some(best_move)
    }
}
