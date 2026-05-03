// Neural network for approximating the q function
use crate::mab::Strategy;
use crate::q_function::QFunction;
use crate::reward::Reward;
use lib::tetris::{BbBoard, BbMove};
use rand::{prelude::SliceRandom, Rng};

const INPUT_SIZE: usize = 400;
const L1_SIZE: usize = 64;
const L2_SIZE: usize = 64;
const OUTPUT_SIZE: usize = 1;

#[derive(Debug)]
pub struct Network {
    w1: [[f64; L1_SIZE]; INPUT_SIZE],
    b1: [f64; L1_SIZE],
    w2: [[f64; L2_SIZE]; L1_SIZE],
    b2: [f64; L2_SIZE],
    w3: [[f64; OUTPUT_SIZE]; L2_SIZE],
    b3: [f64; OUTPUT_SIZE],
}

impl Network {
    pub fn new_empty() -> Self {
        let w1 = [[0.; L1_SIZE]; INPUT_SIZE];
        let b1 = [0.; L1_SIZE];
        let w2 = [[0.; L2_SIZE]; L1_SIZE];
        let b2 = [0.; L2_SIZE];
        let w3 = [[0.; OUTPUT_SIZE]; L2_SIZE];
        let b3 = [0.; OUTPUT_SIZE];

        Network {
            w1,
            b1,
            w2,
            b2,
            w3,
            b3,
        }
    }
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let mut w1 = [[0.; L1_SIZE]; INPUT_SIZE];
        let mut b1 = [0.; L1_SIZE];
        let mut w2 = [[0.; L2_SIZE]; L1_SIZE];
        let mut b2 = [0.; L2_SIZE];
        let mut w3 = [[0.; OUTPUT_SIZE]; L2_SIZE];
        let mut b3 = [0.; OUTPUT_SIZE];
        rng.fill(&mut b1);
        rng.fill(&mut b2);
        rng.fill(&mut b3);
        for w in w1.iter_mut() {
            rng.fill(w);
        }
        for w in w2.iter_mut() {
            rng.fill(w);
        }
        for w in w3.iter_mut() {
            rng.fill(w);
        }

        Network {
            w1,
            b1,
            w2,
            b2,
            w3,
            b3,
        }
    }

    fn input_for(board: &BbBoard) -> [f64; INPUT_SIZE] {
        let mut input = [0.; 400];
        for x in 0..10 {
            for y in 0..40 {
                if board.board[x] & (1 << y) != 0 {
                    input[x * 40 + y] = 1.;
                }
            }
        }
        input
    }

    fn calculate_board(
        &self,
        board: &BbBoard,
    ) -> ([f64; L1_SIZE], [f64; L2_SIZE], [f64; OUTPUT_SIZE]) {
        let input = Self::input_for(board);
        self.calculate(input)
    }

    fn calculate(
        &self,
        input: [f64; INPUT_SIZE],
    ) -> ([f64; L1_SIZE], [f64; L2_SIZE], [f64; OUTPUT_SIZE]) {
        let mut l1 = self.b1;
        let mut l2 = self.b2;
        let mut out = self.b3;
        for (i, l1) in l1.iter_mut().enumerate().take(L1_SIZE) {
            for (j, node) in input.iter().enumerate().take(INPUT_SIZE) {
                *l1 += self.w1[j][i] * node;
            }
        }
        for (i, l2) in l2.iter_mut().enumerate().take(L2_SIZE) {
            for (j, node) in l1.iter().enumerate().take(L1_SIZE) {
                *l2 += self.w2[j][i] * node;
            }
        }
        for (i, out) in out.iter_mut().enumerate().take(OUTPUT_SIZE) {
            for (j, node) in l2.iter().enumerate().take(L2_SIZE) {
                *out += self.w3[j][i] * node;
            }
        }

        (l1, l2, out)
    }

    // The returned network is the gradient of self
    fn gradient(&self, boards: &[BbBoard], learning_rate: f64) -> Network {
        let mut gradient = Network::new_empty();
        let mut rng = rand::thread_rng();

        for iter in 1..=100 {
            let state = boards.choose(&mut rng).unwrap();
            let input = Self::input_for(state);
            let (l1, l2, out) = self.calculate_board(state);
            let real_reward = state.search_for_reward(learning_rate, self);

            // Outer layer
            // Did not multiply by an activation function derivative!
            let delta_out = out[0] - real_reward;
            gradient.b3[0] += (delta_out - gradient.b3[0]) / iter as f64;
            for (i, l2_node) in l2.iter().enumerate().take(L2_SIZE) {
                gradient.w3[i][0] += (delta_out * l2_node - gradient.w3[i][0]) / iter as f64;
            }

            // 2nd layer
            let mut delta_2nd = [0.; L2_SIZE];
            for (j, delta2) in delta_2nd.iter_mut().enumerate().take(L2_SIZE) {
                for i in 0..L2_SIZE {
                    // i dont think this is right
                    *delta2 += self.w3[i][0] * delta_out;
                }
                gradient.b2[j] += (*delta2 - gradient.b2[j]) / iter as f64;
                for (i, l1_node) in l1.iter().enumerate().take(L1_SIZE) {
                    gradient.w2[i][j] += (*delta2 * l1_node - gradient.w2[i][j]) / iter as f64;
                }
            }

            // 1nd layer
            let mut delta_1nd = [0.; L1_SIZE];
            for (j, delta1) in delta_1nd.iter_mut().enumerate().take(L1_SIZE) {
                for i in 0..L1_SIZE {
                    // i dont think this is right either
                    *delta1 += self.w2[i][j] * delta_2nd[j];
                }
                gradient.b1[j] += (*delta1 - gradient.b1[j]) / iter as f64;
                for (i, input_node) in input.iter().enumerate().take(INPUT_SIZE) {
                    gradient.w1[i][j] += (*delta1 * input_node - gradient.w1[i][j]) / iter as f64;
                }
            }
        }

        gradient
    }

    fn update_terms(&mut self, gradient: Network, learning_rate: f64) {
        for i in 0..L1_SIZE {
            self.b1[i] += learning_rate * gradient.b1[i];
            for j in 0..INPUT_SIZE {
                self.w1[j][i] += learning_rate * gradient.w1[j][i];
            }
        }
        for i in 0..L2_SIZE {
            self.b2[i] += learning_rate * gradient.b2[i];
            for j in 0..L1_SIZE {
                self.w2[j][i] += learning_rate * gradient.w2[j][i];
            }
        }
        for i in 0..OUTPUT_SIZE {
            self.b2[i] += learning_rate * gradient.b2[i];
            for j in 0..L2_SIZE {
                self.w2[j][i] += learning_rate * gradient.w2[j][i];
            }
        }
    }

    // Boards are a list of positions to train off of
    pub fn train<S: Strategy>(&mut self, iterations: usize, boards: &[BbBoard], strategy: &mut S) {
        // lol idk how to use this
        let mut rng = rand::thread_rng();
        let learning_rate = 0.1;
        for iter in 0..iterations {
            if iter % 50 == 0 {
                println!("Iteration {}: ", iter);
            }
            let mut state = boards.choose(&mut rng).unwrap().clone();
            loop {
                let moves = state.gen_moves();
                if moves.is_empty() {
                    break;
                }
                let mv = strategy.select(&state, &moves, self).unwrap();
                let new_state = state.make_move(mv);
                let reward = new_state.reward();
                let delta = reward + learning_rate * self.best_score(&new_state).unwrap_or(0.)
                    - self.call(&state, &mv);
                let gradient = self.gradient(boards, learning_rate);
                self.update_terms(gradient, -learning_rate * delta);

                state = new_state;
            }
        }
    }
}

impl QFunction for Network {
    fn call(&self, board: &BbBoard, mv: &BbMove) -> f64 {
        let new_board = board.make_move(*mv);

        self.calculate_board(&new_board).2[0]
    }
}
