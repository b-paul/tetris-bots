use crate::reward::Reward;
use lib::tetris::BbBoard;
use rand::prelude::SliceRandom;

pub trait Simulate {
    fn simulate(&self) -> f64;
}

impl Simulate for BbBoard {
    fn simulate(&self) -> f64 {
        let moves = self.gen_moves();

        if self.queue.is_empty()  || moves.is_empty()
            //|| self.is_dead
        {
            return self.reward();
        }

        let mut rng = rand::thread_rng();
        // Do something better than this! TODO!
        self.make_move(*moves.choose(&mut rng).unwrap()).simulate()
    }
}
