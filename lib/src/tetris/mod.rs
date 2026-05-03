mod col_bbs;
mod naive;
pub use col_bbs::*;
pub use naive::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
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
