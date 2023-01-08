#![allow(unused)]

mod gamestate;
mod moves;
mod settings;
mod utils;

#[macro_use]
extern crate timeit;

mod pieces;

mod multithreading_test;
use multithreading_test::threading_test;

use pieces::Piece;

use crate::pieces::PieceClass;

fn main() {
    //threading_test();
    let piece = Piece {
        class: PieceClass::Pawn,
        enemy: false,
    };
    println!("Char: {}", piece.repr());
}
