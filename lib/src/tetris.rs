extern crate serde;

use crate::TBPBoard;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Piece {
    O,
    I,
    T,
    L,
    J,
    S,
    Z,
    G,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Spin {
    None,
    Mini,
    Full,
}

#[derive(Clone, Copy)]
pub enum Rotation {
    Clockwise,
    AntiClockwise,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Move {
    pub location: Location,
    pub spin: Spin,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Location {
    #[serde(rename = "type")]
    pub piece: Piece,
    pub orientation: Orientation,
    pub x: i8,
    pub y: i8,
}

#[derive(Clone)]
pub struct Board {
    pub hold: Option<Piece>,
    pub queue: Vec<Piece>,
    pub combo: u32,
    pub back_to_back: bool,
    pub board: [[Option<char>; 10]; 40],
}

impl Hash for Move {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = (match self.spin {
            Spin::None => 0,
            Spin::Mini => 1,
            Spin::Full => 2,
        } << 11)
            + (match self.location.orientation {
                Orientation::North => 0,
                Orientation::East => 1,
                Orientation::South => 2,
                Orientation::West => 3,
            } << 9)
            + ((self.location.y as u16) << 4)
            + (self.location.x as u16);
        hash.hash(state);
    }
}

impl Location {
    #[inline]
    fn drop_y(&self, board: &Board) -> i8 {
        // This function is slow and can probably be improved with some bitboard magic
        let mut y = self.y;

        while !board.collision(&Location { y, ..*self }) {
            y -= 1;
        }

        y + 1
    }

    #[inline]
    fn cells(&self) -> [(i8, i8); 4] {
        let mut cells = self.piece.cells(&self.orientation);
        for i in 0..4 {
            cells[i].0 += self.x;
            cells[i].1 += self.y;
        }
        cells
    }

    #[inline]
    fn shift(&self, board: &Board, offset: i8) -> Option<Move> {
        let location = Location {
            x: self.x + offset,
            ..*self
        };
        if board.collision(&location) {
            return None;
        }
        Some(Move {
            location,
            spin: Spin::None,
        })
    }

    #[inline]
    fn rotate(&self, board: &Board, rotation: Rotation) -> Option<Move> {
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

        for i in 0..5 {
            let location = Location {
                x: self.x + srs_table[i].0,
                y: self.y + srs_table[i].1,
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
                return Some(Move { location, spin });
            }
        }
        None
    }

    #[inline]
    fn soft_drop(&self, board: &Board) -> Option<Move> {
        let y = self.drop_y(board);
        if y == self.y {
            return None;
        }
        Some(Move {
            location: Location { y, ..*self },
            spin: Spin::None,
        })
    }
}

impl Board {
    pub fn from_tbp(tbp_board: TBPBoard) -> Self {
        let mut board = [[None; 10]; 40];
        for i in 0..40 {
            for j in 0..10 {
                board[i][j] = tbp_board.board[i][j];
            }
        }
        Board {
            hold: tbp_board.hold,
            queue: tbp_board.queue,
            combo: tbp_board.combo,
            back_to_back: tbp_board.back_to_back,
            board,
        }
    }

    #[inline]
    pub fn occupied(&self, x: i8, y: i8) -> bool {
        !(0..10).contains(&x) || !(0..40).contains(&y) || self.board[y as usize][x as usize] != None
    }

    #[inline]
    pub fn collision(&self, location: &Location) -> bool {
        let cells = location.cells();
        for (x, y) in cells {
            if self.occupied(x, y) {
                return true;
            }
        }
        return false;
    }

    pub fn gen_moves(&self) -> Vec<Move> {
        let mut move_list = Vec::new();
        move_list.append(&mut self.gen_moves_for_piece(self.queue[0]));
        if let Some(hold) = self.hold {
            move_list.append(&mut self.gen_moves_for_piece(hold));
        } else if self.queue.len() >= 2 {
            move_list.append(&mut self.gen_moves_for_piece(self.queue[1]));
        }

        move_list
    }

    pub fn gen_moves_for_piece(&self, piece: Piece) -> Vec<Move> {
        // Performance:
        // Hash set functions account for like 40% of this function REDUCED to like %30 with fxhash
        // drop_y 20%
        // rotate 12%
        // shift 4%
        // soft drop 9%! because it has a drop_y
        // drop_y is slow because of collision

        let mut move_list: Vec<Move> = Vec::with_capacity(64);

        let mut queue = VecDeque::new();
        let mut hash = FxHashSet::default();

        let initial_location = Location {
            piece,
            orientation: Orientation::North,
            x: 5,
            y: 19,
        };

        let initial_move = Move {
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
            move_list.push(Move {
                location: Location { y, ..mv.location },
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

    pub fn make_move(&self, mv: Move) -> Board {
        // I hope making a new board isnt that bad
        // There is like almost a 100% chance this isnt optimal
        // It is very bad.
        // Using column major bitboards means the pext trick shown in cc2 can be used which is way
        // faster than this stuff
        // Using row major bitboards might be faster because you would only have to do 1 or 2 pext
        // instructions and then a bunch of shifts but the shifts might be optimized with simd
        let queue = self.queue.clone();
        let mut new_board = Board { queue, ..*self };
        let piece = mv.location.cells();
        let mut cleared_lines = Vec::new();
        for (x, y) in piece {
            new_board.board[y as usize][x as usize] = Some('T');

            if cleared_lines.contains(&y) {
                continue;
            }

            let mut did_clear = true;
            for x in 0..10 {
                if new_board.board[y as usize][x as usize] == None {
                    did_clear = false;
                    break;
                }
            }
            if did_clear {
                cleared_lines.push(y);
            }
        }
        // Clear line lol this code is so bad
        cleared_lines.sort();
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
            if new_board.hold == None && new_board.queue.len() >= 2 {
                new_board.hold = Some(new_board.queue[0]);
                new_board.queue.remove(0);
            } else {
                new_board.hold = Some(new_board.queue[0]);
            }
        }
        new_board.queue.remove(0);

        new_board
    }

    pub fn make_move_in_place(&mut self, mv: Move) {
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
                    Rotation::Clockwise => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                    Rotation::AntiClockwise => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                },
                Orientation::East => match rotation {
                    Rotation::Clockwise => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                    Rotation::AntiClockwise => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                },
                Orientation::South => match rotation {
                    Rotation::Clockwise => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                    Rotation::AntiClockwise => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                },
                Orientation::West => match rotation {
                    Rotation::Clockwise => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                    Rotation::AntiClockwise => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                },
            },
            Piece::G => panic!("Garbage can't be rotated! What!?!?"),
            Piece::O => [(0, 0); 5],
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
    #[test]
    fn line_clear() {
        // What a lovely looking test!
        let board = crate::Board {
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
            hold: None,
            queue: vec![
                crate::Piece::I,
                crate::Piece::J,
                crate::Piece::O,
                crate::Piece::S,
                crate::Piece::Z,
                crate::Piece::L,
                crate::Piece::T,
            ],
        };
        let mut moves = board.gen_moves(crate::Piece::T);
        moves.sort();
        moves.dedup();
        println!("{:?}", moves);
        let new_board = board.make_move(moves[12]);
        board.print();
        new_board.print();
        assert_eq!(false, board.collision(&moves[0].location));
        assert_eq!(33, moves.len());
    }
}
