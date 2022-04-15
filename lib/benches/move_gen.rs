use criterion::{criterion_group, criterion_main, Criterion};
use tetris_bot_lib::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let board = Board {
        back_to_back: false,
        board: [[None; 10]; 40],
        combo: 0,
        hold: None,
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
    c.bench_function("move gen for empty board I piece", |b| {
        b.iter(|| board.gen_moves(Piece::I))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
