use lib::tetris::BbBoard;

// many ideas from https://github.com/MochBot/fusion/blob/main/src/eval.rs

fn height(bb: u64) -> i64 {
    (64 - bb.leading_zeros()) as i64
}

fn heights(board: &BbBoard) -> [i64; 10] {
    board.board.map(height)
}

/// Number of holes (empty squares with filled squares above them)
fn holes(board: &BbBoard) -> i64 {
    board
        .board
        .map(|bb| {
            let h = height(bb);
            // all squares below (and including) the topmost square in a mask
            let m = (1 << h) - 1;
            (!bb & m).count_ones() as i64
        })
        .into_iter()
        .sum()
}

/// Sum of numbers of filled squares above the topmost hole of each col
fn covered(board: &BbBoard) -> i64 {
    board
        .board
        .map(|bb| {
            // mask of the topmost filled square
            let h = 1u64 << height(bb);
            // mask of just the top hole
            let tm = 1u64 << height((!bb) & (h - 1));

            if tm == 0 {
                0
            } else {
                (h - tm).count_ones() as i64
            }
        })
        .into_iter()
        .sum()
}

/// Find the well column if it exists, which is defined by the code go read it grr
fn well_col_height(board: &BbBoard) -> (Option<usize>, i64) {
    let h = heights(board);
    (0..10).fold((None, 0), |(bc, bd), i| {
        let l = if i == 0 { 40 } else { h[i - 1] };
        let r = if i == 9 { 40 } else { h[i + 1] };
        let h = h[i];
        let d = l.min(r) - h;

        if l > h && r > h && d > bd {
            (Some(i), d)
        } else {
            (bc, bd)
        }
    })
}

/// Sum of differences in height between columns
fn bumpiness(board: &BbBoard, well_col: Option<usize>) -> i64 {
    let heights = heights(board);
    heights
        .iter()
        .zip(heights.iter().skip(1))
        .enumerate()
        .map(|(i, (&h1, &h2))| {
            if Some(i) == well_col || Some(i + 1) == well_col {
                0
            } else {
                h1.abs_diff(h2) as i64
            }
        })
        .sum()
}

/// Sum of squares of differences in height between columns
fn bumpiness_sq(board: &BbBoard, well_col: Option<usize>) -> i64 {
    let heights = heights(board);
    heights
        .iter()
        .zip(heights.iter().skip(1))
        .enumerate()
        .map(|(i, (&h1, &h2))| {
            if Some(i) == well_col || Some(i + 1) == well_col {
                0
            } else {
                (h1.abs_diff(h2) as i64).pow(2)
            }
        })
        .sum()
}

/// Number of differences in fill-ness between horizontally adjacent squares
fn transitions(board: &BbBoard) -> i64 {
    board
        .board
        .iter()
        .zip(board.board.iter().skip(1))
        .map(|(&bb1, &bb2)| (bb1 ^ bb2).count_ones() as i64)
        .sum::<i64>()
        + (0xffffffffff ^ board.board[0]).count_ones() as i64
        + (0xffffffffff ^ board.board[9]).count_ones() as i64
}

pub fn eval(board: &BbBoard) -> i64 {
    let dead = (3..=6).any(|i| board.board[i] & (1 << 19) != 0);
    if dead {
        return -999999999;
    }

    if board.broke_b2b {
        //return -111111111;
    }

    let (well_col, _well_height) = well_col_height(board);

    let height = heights(board).into_iter().max().unwrap();
    let holes = holes(board);
    let covered = covered(board);
    let bumpiness = bumpiness(board, well_col);
    let bumpiness_sq = bumpiness_sq(board, well_col);
    let transitions = transitions(board);

    let height_diff = height * -2
        + if height > 10 { (height - 10) * -10 } else { 0 }
        + if height > 15 { (height - 15) * -50 } else { 0 };

    let b2b = board.back_to_back_count as i64;

    height_diff + holes * -80 + covered * -10 + bumpiness * -3 - bumpiness_sq
        + transitions * -3
        + b2b.min(10) * 100
}
