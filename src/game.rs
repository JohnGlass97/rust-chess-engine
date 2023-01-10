use std::time::Instant;

use crate::{
    gamestate::{self, GameState},
    moves::Move,
    simulation::analyse,
};

fn find_best_development(game_state: &GameState, mut moves: Vec<Move>) -> Move {
    moves.remove(0)
}

pub fn find_best_move(game_state: &GameState, recurision_depth: i8) -> Option<Move> {
    let timer = Instant::now();
    let analysis = analyse(&game_state, recurision_depth, true);

    if analysis.score > 250 {
        println!("Checkmate found!");
    } else if analysis.engine_no_moves {
        println!("No moves found, game over?");
        return None;
    }

    let mut moves = analysis.best_moves;
    let best_move = find_best_development(game_state, moves);

    game_state.perform_move(&best_move).print();

    println!(
        "{} selected from {} tested moves.",
        best_move.repr(),
        analysis.tested_moves
    );
    println!(
        "Simulatesd {} moves, took {} seconds or {} ms",
        analysis.sim_moves,
        timer.elapsed().as_secs(),
        timer.elapsed().as_millis()
    );
    Some(best_move)
}
