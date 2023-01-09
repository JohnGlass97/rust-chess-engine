#![allow(unused)]

mod gamestate;
mod moves;
mod pieces;
mod settings;
mod utils;

#[macro_use]
extern crate timeit;

mod multithreading_test;
use gamestate::parse_layout;
use moves::{Move, MoveType};
use multithreading_test::threading_test;

use pieces::Piece;

use crate::pieces::PieceClass;

const DEFAULT_LAYOUT: &str = "
R N B Q K B N R
P P P P P P P P
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
p p p p p p p p
r n b q k b n r
";

fn main() {
    let mut game_state = parse_layout(DEFAULT_LAYOUT);
    game_state = game_state.mov(&Move {
        enemy: true,
        move_type: MoveType::Castling(true),
    });
    game_state.print();
}
