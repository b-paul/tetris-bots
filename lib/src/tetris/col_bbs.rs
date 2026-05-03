// Column wise bitborads
extern crate serde;

use crate::{Board, Move, Orientation, Piece, Rotation, Spin, TBPBoard, TBPLocation, TBPMove};
use rand::prelude::SliceRandom;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Clone)]
pub struct BbBoard {
    pub hold: Option<Piece>,
    pub queue: Vec<Piece>,
    pub combo: u32,
    pub back_to_back: bool,
    pub back_to_back_count: usize,
    pub board: [u64; 10],
    pub tspins: usize,
    pub wasted_ts: usize,
    /// for silly bots that want to never break b2b
    pub broke_b2b: bool,
    pub minied: bool,
    // counter for single, double, triple, tetris, tss, tsd and tst clears
}

impl Board for BbBoard {
    fn from_tbp(tbp_board: TBPBoard) -> Self {
        let mut board = [0; 10];
        for (r, row) in tbp_board.board.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if cell.is_some() {
                    board[c] |= 1 << r;
                }
            }
        }

        BbBoard {
            hold: tbp_board.hold,
            queue: tbp_board.queue,
            combo: tbp_board.combo,
            back_to_back: tbp_board.back_to_back,
            back_to_back_count: if tbp_board.back_to_back { 1 } else { 0 },
            board,
            tspins: 0,
            wasted_ts: 0,
            broke_b2b: false,
            minied: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct BbMove {
    pub location: BbLocation,
    pub spin: Spin,
}

impl From<TBPMove> for BbMove {
    fn from(tbp: TBPMove) -> Self {
        BbMove {
            location: tbp.location.into(),
            spin: tbp.spin,
        }
    }
}

impl From<BbMove> for TBPMove {
    fn from(mv: BbMove) -> Self {
        let spin = match mv.location.piece {
            Piece::T => mv.spin,
            _ => Spin::None,
        };
        TBPMove {
            location: mv.location.into(),
            spin,
        }
    }
}

impl Move for BbMove {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct BbLocation {
    #[serde(rename = "type")]
    pub piece: Piece,
    pub orientation: Orientation,
    pub x: i8,
    pub y: i8,
}

impl From<BbLocation> for TBPLocation {
    fn from(loc: BbLocation) -> Self {
        TBPLocation {
            piece: loc.piece,
            orientation: loc.orientation,
            x: loc.x,
            y: loc.y,
        }
    }
}

impl From<TBPLocation> for BbLocation {
    fn from(tbp: TBPLocation) -> Self {
        BbLocation {
            piece: tbp.piece,
            orientation: tbp.orientation,
            x: tbp.x,
            y: tbp.y,
        }
    }
}

impl BbLocation {
    #[inline]
    fn drop_y(&self, board: &BbBoard) -> i8 {
        let drop_dist = self
            .cells()
            .iter()
            .map(|(x, y)| {
                // REFACTOR
                // This is kinda ugly
                if *y == 0 {
                    return 0;
                }
                let shift = 64 - y;
                ((!board.board[*x as usize]) << shift).leading_ones() as i8
            })
            .min()
            .unwrap();
        self.y - drop_dist
    }

    #[inline]
    fn cells(&self) -> [(i8, i8); 4] {
        let mut cells = self.piece.cells(&self.orientation);
        for cell in &mut cells {
            cell.0 += self.x;
            cell.1 += self.y;
        }
        cells
    }

    #[inline]
    fn shift(&self, board: &BbBoard, offset: i8) -> Option<BbMove> {
        let location = BbLocation {
            x: self.x + offset,
            ..*self
        };
        if board.collision(&location) {
            return None;
        }
        Some(BbMove {
            location,
            spin: Spin::None,
        })
    }

    /// determines wheter shifting the location by the given offset is blocked
    fn shift_blocked(&self, board: &BbBoard, (dx, dy): (i8, i8)) -> bool {
        let location = BbLocation {
            x: self.x + dx,
            y: self.y + dy,
            ..*self
        };
        board.collision(&location)
    }

    #[inline]
    fn rotate(&self, board: &BbBoard, rotation: Rotation) -> Option<BbMove> {
        if self.piece == Piece::O {
            return None;
        }

        let srs_table = self.piece.srs_table(&self.orientation, rotation);

        let orientation = match self.orientation {
            Orientation::North => match rotation {
                Rotation::Clockwise => Orientation::East,
                Rotation::AntiClockwise => Orientation::West,
            },
            Orientation::East => match rotation {
                Rotation::Clockwise => Orientation::South,
                Rotation::AntiClockwise => Orientation::North,
            },
            Orientation::South => match rotation {
                Rotation::Clockwise => Orientation::West,
                Rotation::AntiClockwise => Orientation::East,
            },
            Orientation::West => match rotation {
                Rotation::Clockwise => Orientation::North,
                Rotation::AntiClockwise => Orientation::South,
            },
        };

        for (i, entry) in srs_table.iter().enumerate() {
            let location = BbLocation {
                x: self.x + entry.0,
                y: self.y + entry.1,
                orientation,
                ..*self
            };
            if !board.collision(&location) {
                let mut spin = Spin::None;
                if location.piece == Piece::T {
                    let corners = [(1, 1), (-1, 1), (1, -1), (-1, -1)]
                        .into_iter()
                        .filter(|(x, y)| board.occupied(location.x + x, location.y + y))
                        .count();
                    let mini_corners = match orientation {
                        Orientation::North => [(-1, 1), (1, 1)],
                        Orientation::East => [(1, 1), (1, -1)],
                        Orientation::South => [(1, -1), (-1, -1)],
                        Orientation::West => [(-1, -1), (-1, 1)],
                    }
                    .into_iter()
                    .filter(|(x, y)| board.occupied(location.x + x, location.y + y))
                    .count();
                    if corners >= 3 {
                        if mini_corners == 2 || i == 4 {
                            spin = Spin::Full;
                        } else {
                            spin = Spin::Mini;
                        }
                    }
                } else {
                    // piece can't move in any direction -> mini
                    const DIRS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                    //const DIRS: [(i8, i8); 1] = [(0, 1)];
                    if DIRS.iter().all(|&off| location.shift_blocked(board, off)) {
                        spin = Spin::Mini;
                    }
                }
                return Some(BbMove { location, spin });
            }
        }
        None
    }

    #[inline]
    fn soft_drop(&self, board: &BbBoard) -> Option<BbMove> {
        let y = self.drop_y(board);
        if y == self.y {
            return None;
        }
        Some(BbMove {
            location: BbLocation { y, ..*self },
            spin: Spin::None,
        })
    }
}

impl BbBoard {
    #[inline]
    pub fn occupied(&self, x: i8, y: i8) -> bool {
        !(0..10).contains(&x) || !(0..40).contains(&y) || (self.board[x as usize] & (1 << y)) != 0
    }

    #[inline]
    pub fn collision(&self, location: &BbLocation) -> bool {
        location.cells().iter().any(|&(x, y)| self.occupied(x, y))
    }

    pub fn gen_moves(&self) -> Vec<BbMove> {
        if self.queue.is_empty() {
            return vec![];
        }

        let mut move_list = Vec::with_capacity(128);
        move_list.append(&mut self.gen_moves_for_piece(self.queue[0]));
        if let Some(hold) = self.hold {
            move_list.append(&mut self.gen_moves_for_piece(hold));
        } else if self.queue.len() >= 2 {
            move_list.append(&mut self.gen_moves_for_piece(self.queue[1]));
        }

        move_list
    }

    pub fn gen_moves_for_piece(&self, piece: Piece) -> Vec<BbMove> {
        // Performance:
        // Hash set functions account for like 40% of this function REDUCED to like %30 with fxhash
        // drop_y 20%
        // rotate 12%
        // shift 4%
        // soft drop 9%! because it has a drop_y
        // drop_y is slow because of collision

        let mut move_list: Vec<BbMove> = Vec::with_capacity(64);

        let mut stack = Vec::new();
        let mut hash = FxHashSet::default();
        let mut move_list_hashes = FxHashSet::default();

        let mut initial_location = BbLocation {
            piece,
            orientation: Orientation::North,
            x: 5,
            y: 19,
        };

        if self.collision(&initial_location) {
            initial_location.y += 1;
            if self.collision(&initial_location) {
                return vec![];
            }
        }

        let initial_move = BbMove {
            location: initial_location,
            spin: Spin::None,
        };

        stack.push(initial_move);
        hash.insert(initial_move);

        // DFS

        while let Some(mv) = stack.pop() {
            let y = mv.location.drop_y(self);
            let mut spin = Spin::None;
            if y == mv.location.y {
                spin = mv.spin;
            }
            let placed_move = BbMove {
                location: BbLocation { y, ..mv.location },
                spin,
            };
            move_list_hashes.insert(placed_move);

            // Look at each action from this position
            if let Some(mv) = mv.location.shift(self, -1) {
                if !hash.contains(&mv) {
                    stack.push(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.shift(self, 1) {
                if !hash.contains(&mv) {
                    stack.push(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.rotate(self, Rotation::Clockwise) {
                if !hash.contains(&mv) {
                    stack.push(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.rotate(self, Rotation::AntiClockwise) {
                if !hash.contains(&mv) {
                    stack.push(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.soft_drop(self) {
                if !hash.contains(&mv) {
                    stack.push(mv);
                    hash.insert(mv);
                }
            }
        }

        move_list.extend(move_list_hashes.iter());
        move_list
    }

    fn set_sq(&mut self, x: i8, y: i8) {
        debug_assert!((0..10).contains(&x));
        self.board[x as usize] |= 1 << y;
    }

    // oh my god! undo is so much better! probably
    pub fn make_move(&self, mv: BbMove) -> BbBoard {
        // Set the squares then clear the lines
        let queue = self.queue.clone();
        let mut new_board = BbBoard { queue, ..*self };

        // Set the squares
        for (x, y) in mv.location.cells() {
            new_board.set_sq(x, y);
        }

        // Clear lines
        let clear_mask = new_board.board.iter().fold(!0u64, |a, b| a & b);
        let mut lines_cleared = 0;

        if clear_mask != 0 {
            lines_cleared = clear_mask.count_ones();
            for col in new_board.board.iter_mut() {
                // Preferably implement a no pext version (im on zen2 so no pext might even be
                // faster lol)
                *col = unsafe { std::arch::x86_64::_pext_u64(*col, !clear_mask) };
            }
        }

        // Update the queue
        if !new_board.queue.is_empty() {
            if mv.location.piece != new_board.queue[0] {
                if new_board.hold.is_none() && new_board.queue.len() >= 2 {
                    new_board.hold = Some(new_board.queue[0]);
                    new_board.queue.remove(0);
                } else {
                    new_board.hold = Some(new_board.queue[0]);
                }
            }
            new_board.queue.remove(0);
        }

        // Count spins
        if mv.location.piece == Piece::T {
            if lines_cleared == 0 {
                new_board.wasted_ts += 1;
            } else if mv.spin == Spin::Full {
                // Have individual counters for each spin
                new_board.tspins += lines_cleared as usize;
            } else {
                // Add line clear counter
            }
        } else {
            // add line clear counter
        }

        if lines_cleared != 0 {
            if lines_cleared == 4 || mv.spin != Spin::None {
                new_board.back_to_back = true;
                new_board.back_to_back_count += 1;
            } else {
                new_board.broke_b2b = true;
                new_board.back_to_back = false;
                new_board.back_to_back_count = 0;
            }
        }

        new_board.minied = mv.spin == Spin::Mini;

        new_board
    }

    pub fn make_move_in_place(&mut self, mv: BbMove) {
        let board = self.make_move(mv);
        *self = board;
    }

    // Update the queue to be of length len and fill in the queue wih random pieces
    pub fn make_queue_len(&mut self, len: usize) {
        let mut rng = rand::thread_rng();
        let mut bag = vec![];
        while self.queue.len() < len {
            if bag.is_empty() {
                bag = vec![
                    Piece::O,
                    Piece::I,
                    Piece::T,
                    Piece::L,
                    Piece::J,
                    Piece::S,
                    Piece::Z,
                ];
            }
            let piece = bag.choose(&mut rng).unwrap();
            self.queue.push(*piece);
        }
    }

    pub fn print(&self) {
        for i in (0..40).rev() {
            let mut str = String::new();
            for j in 0..10 {
                let char = if self.board[j] & (1 << i) != 0 {
                    '#'
                } else {
                    '.'
                };
                str.insert(j, char);
            }
            println!("{}", str);
        }
    }

    pub fn perft(self) -> usize {
        let p = self.queue[0];

        if self.queue.len() == 1 {
            println!("{:?}", self.gen_moves_for_piece(p));
            self.gen_moves_for_piece(p).len()
        } else {
            self.gen_moves_for_piece(p)
                .into_iter()
                .map(|m| self.make_move(m).perft())
                .sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft() {
        let queue = vec![
            Piece::I,
        ];
        let board = BbBoard {
            hold: None,
            queue,
            combo: 0,
            back_to_back: false,
            back_to_back_count: 0,
            //board: [0; 10],
            board: [1,1,3,3,1,0,0,0,14,15],
            tspins: 0,
            wasted_ts: 0,
            broke_b2b: false,
            minied: false,
        };
        assert_eq!(board.perft(), 17)
    }
}
