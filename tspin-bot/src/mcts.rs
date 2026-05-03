use crate::mab::{Strategy, UCB1};
use crate::q_function::QFunction;
use crate::reward::Reward;
use crate::simulate::Simulate;
use lib::tetris::{BbBoard, BbMove};
use lib::SearchStatus;
use rand::prelude::SliceRandom;
use std::collections::HashMap;

#[derive(PartialEq)]
enum NodeState {
    Expanded,
    Expandable,
    Leaf,
}

struct Node {
    board: BbBoard,
    children: HashMap<BbMove, Node>,
    state: NodeState,

    num: usize,
    q: f64,
}

impl Node {
    fn new(board: BbBoard) -> Self {
        Node {
            board,
            children: HashMap::new(),
            state: NodeState::Expandable,

            num: 0,
            q: 0.,
        }
    }

    fn build_root_from(board: BbBoard) -> Self {
        let mut children = HashMap::new();
        let moves = board.gen_moves();
        for mv in moves {
            children.insert(mv, Node::new(board.make_move(mv)));
        }
        Node {
            board,
            children,
            state: NodeState::Expanded,

            num: 0,
            q: 0.,
        }
    }

    // Returns an unexpanded node
    #[allow(dead_code)]
    fn search<Q: QFunction, S: Strategy>(
        &mut self,
        board: &mut BbBoard,
        q_function: &Q,
        strategy: &mut S,
    ) -> &mut Node {
        let mut node = self;
        while node.state == NodeState::Expanded {
            let moves = node.board.gen_moves();
            // :grimacing:
            let mv = strategy.select(board, &moves, q_function).unwrap();
            node = node.children.get_mut(&mv).unwrap();
        }
        node
    }

    fn expand(&mut self) -> Option<(&mut Node, BbMove)> {
        let moves = self.board.gen_moves();
        if moves.is_empty() {
            self.state = NodeState::Leaf;
            return None;
        }
        let mut rng = rand::thread_rng();

        let mut moves_left = vec![];
        for mv in moves {
            if !self.children.contains_key(&mv) {
                moves_left.push(mv);
            }
        }

        if moves_left.len() == 1 {
            self.state = NodeState::Expanded;
        }

        // Do something better than this!! TODO!!!
        let mv = moves_left.choose(&mut rng)?;

        let node = Node::new(self.board.make_move(*mv));
        self.children.insert(*mv, node);
        self.children.get_mut(mv).map(|node| (node, *mv))
    }

    fn simulate(&mut self, mv: BbMove) -> f64 {
        let board = self.board.make_move(mv);
        let g = board.simulate();

        self.num += 1;
        self.q += (g - self.q) / self.num as f64;
        g
    }

    fn reward(&self) -> f64 {
        self.board.reward()
    }

    // One day maybe this will be implemented (non recursive version of iteration)
    //fn backpropagate(&mut self, _reward: f64) {
    // Yikes this has to go up to the parent and stuff
    //todo!()
    //}

    fn iteration<Q: QFunction, S: Strategy>(&mut self, strategy: &mut S, q_function: &Q) -> f64 {
        // TODO Move the node counter here!
        let g = match self.state {
            NodeState::Leaf => {
                // reward
                self.reward()
            }
            NodeState::Expandable => {
                let node = self.expand();
                match node {
                    Some((node, mv)) => node.simulate(mv),
                    None => self.reward(),
                }
            }
            NodeState::Expanded => {
                let moves = self.board.gen_moves();
                let mv = strategy.select(&self.board, &moves, q_function).unwrap();
                let node = self.children.get_mut(&mv).unwrap();
                node.iteration(strategy, q_function)
            }
        };

        self.num += 1;
        self.q += (g - self.q) / self.num as f64;
        g
    }

    fn best_move(&self) -> BbMove {
        let moves = self.board.gen_moves();
        let mut best_move = moves[0];
        let mut best_score = -999999.;
        for mv in moves {
            let score = self.children.get(&mv).unwrap().q;
            if score > best_score {
                best_move = mv;
                best_score = score;
            }
        }
        best_move
    }
}

pub fn mcts<Q: QFunction>(
    status: &SearchStatus<BbMove>,
    board: &mut BbBoard,
    q_function: &Q,
) -> BbMove {
    let mut root_node = Node::build_root_from(board.clone());
    let mut strategy = UCB1::new();
    let mut nodes = 0;
    loop {
        if !status.terminate() && nodes > 10000 {
            break;
        }
        nodes += 1;
        root_node.iteration(&mut strategy, q_function);
    }

    root_node.best_move()
}

pub fn internal_mcts<Q: QFunction>(
    board: &mut BbBoard,
    q_function: &Q,
) -> BbMove {
    let mut root_node = Node::build_root_from(board.clone());
    let mut strategy = UCB1::new();
    let mut nodes = 0;
    loop {
        if nodes > 100 {
            break;
        }
        nodes += 1;
        root_node.iteration(&mut strategy, q_function);
    }

    root_node.best_move()
}
