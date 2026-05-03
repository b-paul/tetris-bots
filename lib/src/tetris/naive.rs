extern crate serde;

use crate::{Board, Move, Orientation, Piece, Rotation, Spin, TBPBoard, TBPMove, TBPLocation};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct NaiveMove {
    pub location: NaiveLocation,
    pub spin: Spin,
}

impl From<NaiveMove> for TBPMove {
    fn from(mv: NaiveMove) -> Self {
        TBPMove {
            location: mv.location.into(),
            spin: mv.spin,
        }
    }
}

impl From<TBPMove> for NaiveMove {
    fn from(tbp: TBPMove) -> Self {
        NaiveMove {
            location: tbp.location.into(),
            spin: tbp.spin,
        }
    }
}

impl Move for NaiveMove {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct NaiveLocation {
    #[serde(rename = "type")]
    pub piece: Piece,
    pub orientation: Orientation,
    pub x: i8,
    pub y: i8,
}

impl From<NaiveLocation> for TBPLocation {
    fn from(loc: NaiveLocation) -> Self {
        TBPLocation {
            piece: loc.piece,
            orientation: loc.orientation,
            x: loc.x,
            y: loc.y,
        }
    }
}


impl From<TBPLocation> for NaiveLocation {
    fn from(tbp: TBPLocation) -> Self {
        NaiveLocation {
            piece: tbp.piece,
            orientation: tbp.orientation,
            x: tbp.x,
            y: tbp.y,
        }
    }
}

#[derive(Clone)]
pub struct NaiveBoard {
    pub hold: Option<Piece>,
    pub queue: Vec<Piece>,
    pub combo: u32,
    pub back_to_back: bool,
    pub board: [[Option<char>; 10]; 40],
}

impl Board for NaiveBoard {
    fn from_tbp(tbp_board: TBPBoard) -> Self {
        let mut board = [[None; 10]; 40];
        for (i, row) in board.iter_mut().enumerate() {
            row[..10].copy_from_slice(&tbp_board.board[i][..10]);
        }
        NaiveBoard {
            hold: tbp_board.hold,
            queue: tbp_board.queue,
            combo: tbp_board.combo,
            back_to_back: tbp_board.back_to_back,
            board,
        }
    }
}

impl NaiveLocation {
    #[inline]
    fn drop_y(&self, board: &NaiveBoard) -> i8 {
        // This function is slow and can probably be improved with some bitboard magic
        let mut y = self.y;

        while !board.collision(&NaiveLocation { y, ..*self }) {
            y -= 1;
        }

        y + 1
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
    fn shift(&self, board: &NaiveBoard, offset: i8) -> Option<NaiveMove> {
        let location = NaiveLocation {
            x: self.x + offset,
            ..*self
        };
        if board.collision(&location) {
            return None;
        }
        Some(NaiveMove {
            location,
            spin: Spin::None,
        })
    }

    #[inline]
    fn rotate(&self, board: &NaiveBoard, rotation: Rotation) -> Option<NaiveMove> {
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
            let location = NaiveLocation {
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
                }
                return Some(NaiveMove { location, spin });
            }
        }
        None
    }

    #[inline]
    fn soft_drop(&self, board: &NaiveBoard) -> Option<NaiveMove> {
        let y = self.drop_y(board);
        if y == self.y {
            return None;
        }
        Some(NaiveMove {
            location: NaiveLocation { y, ..*self },
            spin: Spin::None,
        })
    }
}

impl NaiveBoard {
    #[inline]
    pub fn occupied(&self, x: i8, y: i8) -> bool {
        !(0..10).contains(&x) || !(0..40).contains(&y) || self.board[y as usize][x as usize].is_some()
    }

    #[inline]
    pub fn collision(&self, location: &NaiveLocation) -> bool {
        let cells = location.cells();
        for (x, y) in cells {
            if self.occupied(x, y) {
                return true;
            }
        }
        false
    }

    pub fn gen_moves(&self) -> Vec<NaiveMove> {
        let mut move_list = Vec::new();
        move_list.append(&mut self.gen_moves_for_piece(self.queue[0]));
        if let Some(hold) = self.hold {
            move_list.append(&mut self.gen_moves_for_piece(hold));
        } else if self.queue.len() >= 2 {
            move_list.append(&mut self.gen_moves_for_piece(self.queue[1]));
        }

        move_list
    }

    pub fn gen_moves_for_piece(&self, piece: Piece) -> Vec<NaiveMove> {
        // Performance:
        // Hash set functions account for like 40% of this function REDUCED to like %30 with fxhash
        // drop_y 20%
        // rotate 12%
        // shift 4%
        // soft drop 9%! because it has a drop_y
        // drop_y is slow because of collision

        let mut move_list: Vec<NaiveMove> = Vec::with_capacity(64);

        let mut queue = VecDeque::new();
        let mut hash = FxHashSet::default();

        let initial_location = NaiveLocation {
            piece,
            orientation: Orientation::North,
            x: 5,
            y: 19,
        };

        let initial_move = NaiveMove {
            location: initial_location,
            spin: Spin::None,
        };

        queue.push_back(initial_move);
        hash.insert(initial_move);

        // BFS

        while let Some(mv) = queue.pop_front() {
            let y = mv.location.drop_y(self);
            let mut spin = Spin::None;
            if y == mv.location.y {
                spin = mv.spin;
            }
            move_list.push(NaiveMove {
                location: NaiveLocation { y, ..mv.location },
                spin,
            });

            // Look at each action from this position
            if let Some(mv) = mv.location.shift(self, -1) {
                if !hash.contains(&mv) {
                    queue.push_back(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.shift(self, 1) {
                if !hash.contains(&mv) {
                    queue.push_back(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.rotate(self, Rotation::Clockwise) {
                if !hash.contains(&mv) {
                    queue.push_back(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.rotate(self, Rotation::AntiClockwise) {
                if !hash.contains(&mv) {
                    queue.push_back(mv);
                    hash.insert(mv);
                }
            }
            if let Some(mv) = mv.location.soft_drop(self) {
                if !hash.contains(&mv) {
                    queue.push_back(mv);
                    hash.insert(mv);
                }
            }
        }

        move_list.sort();
        move_list.dedup();

        move_list
    }

    pub fn make_move(&self, mv: NaiveMove) -> NaiveBoard {
        // I hope making a new board isnt that bad
        // There is like almost a 100% chance this isnt optimal
        // It is very bad.
        // Using column major bitboards means the pext trick shown in cc2 can be used which is way
        // faster than this stuff
        // Using row major bitboards might be faster because you would only have to do 1 or 2 pext
        // instructions and then a bunch of shifts but the shifts might be optimized with simd
        let queue = self.queue.clone();
        let mut new_board = NaiveBoard { queue, ..*self };
        let piece = mv.location.cells();
        let mut cleared_lines = Vec::new();
        for (x, y) in piece {
            // t????
            new_board.board[y as usize][x as usize] = Some('T');

            if cleared_lines.contains(&y) {
                continue;
            }

            let mut did_clear = true;
            for x in 0..10 {
                if new_board.board[y as usize][x as usize].is_none() {
                    did_clear = false;
                    break;
                }
            }
            if did_clear {
                cleared_lines.push(y);
            }
        }
        // Clear line lol this code is so bad
        cleared_lines.sort_unstable();
        cleared_lines.reverse();
        for line in cleared_lines {
            for y in line..39 {
                for x in 0..10 {
                    new_board.board[y as usize][x as usize] =
                        new_board.board[(y + 1) as usize][x as usize];
                }
            }
            for x in 0..10 {
                new_board.board[39][x as usize] = None;
            }
        }

        // Update the queue
        if mv.location.piece != new_board.queue[0] {
            if new_board.hold.is_none() && new_board.queue.len() >= 2 {
                new_board.hold = Some(new_board.queue[0]);
                new_board.queue.remove(0);
            } else {
                new_board.hold = Some(new_board.queue[0]);
            }
        }
        new_board.queue.remove(0);

        new_board
    }

    pub fn make_move_in_place(&mut self, mv: NaiveMove) {
        let board = self.make_move(mv);
        *self = board;
    }

    pub fn print(&self) {
        for i in (0..40).rev() {
            let mut str = String::new();
            for j in 0..10 {
                let char = self.board[i][j].unwrap_or('.');
                str.insert(j, char);
            }
            println!("{}", str);
        }
    }
}

impl Piece {
    #[inline]
    pub fn cells(&self, orientation: &Orientation) -> [(i8, i8); 4] {
        // What a lovely looking function!
        // Should make a macro out of this tbh
        match self {
            Piece::O => match orientation {
                Orientation::North => [(0, 0), (1, 0), (0, 1), (1, 1)],
                Orientation::East => [(0, 0), (1, 0), (0, -1), (1, -1)],
                Orientation::South => [(0, 0), (-1, 0), (0, -1), (-1, -1)],
                Orientation::West => [(0, 0), (-1, 0), (0, 1), (-1, 1)],
            },
            Piece::I => match orientation {
                Orientation::North => [(0, 0), (-1, 0), (1, 0), (2, 0)],
                Orientation::East => [(0, 0), (0, 1), (0, -1), (0, -2)],
                Orientation::South => [(0, 0), (1, 0), (-1, 0), (-2, 0)],
                Orientation::West => [(0, 0), (0, -1), (0, 1), (0, 2)],
            },
            Piece::T => match orientation {
                Orientation::North => [(0, 0), (1, 0), (0, 1), (-1, 0)],
                Orientation::East => [(0, 0), (0, 1), (1, 0), (0, -1)],
                Orientation::South => [(0, 0), (1, 0), (0, -1), (-1, 0)],
                Orientation::West => [(0, 0), (0, 1), (-1, 0), (0, -1)],
            },
            Piece::L => match orientation {
                Orientation::North => [(0, 0), (-1, 0), (1, 0), (1, 1)],
                Orientation::East => [(0, 0), (0, 1), (0, -1), (1, -1)],
                Orientation::South => [(0, 0), (1, 0), (-1, 0), (-1, -1)],
                Orientation::West => [(0, 0), (0, -1), (0, 1), (-1, 1)],
            },
            Piece::J => match orientation {
                Orientation::North => [(0, 0), (-1, 0), (1, 0), (-1, 1)],
                Orientation::East => [(0, 0), (0, 1), (0, -1), (1, 1)],
                Orientation::South => [(0, 0), (1, 0), (-1, 0), (1, -1)],
                Orientation::West => [(0, 0), (0, -1), (0, 1), (-1, -1)],
            },
            Piece::S => match orientation {
                Orientation::North => [(0, 0), (-1, 0), (0, 1), (1, 1)],
                Orientation::East => [(0, 0), (0, 1), (1, 0), (1, -1)],
                Orientation::South => [(0, 0), (1, 0), (0, -1), (-1, -1)],
                Orientation::West => [(0, 0), (0, -1), (-1, 0), (-1, 1)],
            },
            Piece::Z => match orientation {
                Orientation::North => [(0, 0), (1, 0), (0, 1), (-1, 1)],
                Orientation::East => [(0, 0), (0, -1), (1, 0), (1, 1)],
                Orientation::South => [(0, 0), (-1, 0), (0, -1), (1, -1)],
                Orientation::West => [(0, 0), (0, 1), (-1, 0), (-1, -1)],
            },
            Piece::G => panic!("Garbage can't be placed! What!?!?"),
        }
    }

    #[inline]
    pub fn srs_table(&self, orientation: &Orientation, rotation: Rotation) -> [(i8, i8); 5] {
        match self {
            Piece::I => match orientation {
                Orientation::North => match rotation {
                    Rotation::Clockwise => [(1, 0), (-1, 0), (2, 0), (-1, -1), (2, 2)],
                    Rotation::AntiClockwise => [(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],
                },
                Orientation::East => match rotation {
                    Rotation::Clockwise => [(0, -1), (-1, -1), (2, -1), (-1, 1), (2, -2)],
                    Rotation::AntiClockwise => [(-1, 0), (1, 0), (-2, 0), (1, 1), (-2, -2)],
                },
                Orientation::South => match rotation {
                    Rotation::Clockwise => [(-1, 0), (1, 0), (-2, 0), (1, 1), (-2, -2)],
                    Rotation::AntiClockwise => [(0, 1), (1, 1), (-2, 1), (1, -1), (-2, 2)],
                },
                Orientation::West => match rotation {
                    Rotation::Clockwise => [(0, 1), (1, 1), (-2, 1), (1, -1), (-2, 2)],
                    Rotation::AntiClockwise => [(1, 0), (-1, 0), (2, 0), (-1, -1), (2, 2)],
                },
            },
            Piece::G => panic!("Garbage can't be rotated! What!?!?"),
            Piece::O => match orientation {
                Orientation::North => match rotation {
                    Rotation::Clockwise => [(0, 1); 5],
                    Rotation::AntiClockwise => [(1, 0); 5],
                },
                Orientation::East => match rotation {
                    Rotation::Clockwise => [(1, 0); 5],
                    Rotation::AntiClockwise => [(0, -1); 5],
                },
                Orientation::South => match rotation {
                    Rotation::Clockwise => [(0, -1); 5],
                    Rotation::AntiClockwise => [(-1, 0); 5],
                },
                Orientation::West => match rotation {
                    Rotation::Clockwise => [(-1, 0); 5],
                    Rotation::AntiClockwise => [(0, 1); 5],
                },
            },
            _ => match orientation {
                Orientation::North => match rotation {
                    Rotation::Clockwise => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                    Rotation::AntiClockwise => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                },
                Orientation::East => match rotation {
                    Rotation::Clockwise => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                    Rotation::AntiClockwise => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                },
                Orientation::South => match rotation {
                    Rotation::Clockwise => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                    Rotation::AntiClockwise => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                },
                Orientation::West => match rotation {
                    Rotation::Clockwise => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                    Rotation::AntiClockwise => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn line_clear() {
        // What a lovely looking test!
        let board = NaiveBoard {
            back_to_back: false,
            board: [
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [None; 10],
                [
                    Some('G'),
                    Some('G'),
                    Some('G'),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
                [None; 10],
            ],
            combo: 0,
            hold: Some(Piece::T),
            queue: vec![
                Piece::I,
                Piece::J,
                Piece::O,
                Piece::S,
                Piece::Z,
                Piece::L,
                Piece::T,
            ],
        };
        let mut moves = board.gen_moves();
        moves.sort();
        moves.dedup();
        println!("{:?}", moves);
        let new_board = board.make_move(moves[12]);
        board.print();
        new_board.print();
        assert!(!board.collision(&moves[0].location));
    }
}
