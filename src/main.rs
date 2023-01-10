#![allow(unused)]

mod game;
mod gamestate;
mod moves;
mod pieces;
mod settings;
mod simulation;
mod utils;

use std::time::Instant;

use game::find_best_move;
use gamestate::parse_layout;
use simulation::analyse;
use utils::AnalysisResult;

fn main() {
    let game_state = parse_layout();
    find_best_move(&game_state, 3);
}
