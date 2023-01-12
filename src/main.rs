#![allow(unused)]

mod development;
mod gamestate;
mod moves;
mod pieces;
mod settings;
mod simulation;
mod utils;

use std::time::Instant;

use gamestate::{parse_layout, GameState};
use moves::{input_move, Move};
use settings::GAME_LOOP;
use simulation::analyse;
use utils::input;

use crate::development::find_best_development;

fn find_best_move(game_state: &GameState, recurision_depth: u8) -> Option<Move> {
    game_state.print_direct();

    let timer = Instant::now();
    let analysis = analyse(&game_state, recurision_depth, true);

    let moves = analysis.best_moves.unwrap();

    for mov in moves.iter() {
        assert!(!mov.enemy);
        println!("{}", mov.repr());
    }

    if analysis.score_buffer[0] > 250 {
        println!("Checkmate found!");
    } else if analysis.engine_no_moves {
        println!("No moves found, game over?");
        return None;
    } else {
        println!("Best score: {}", analysis.score_buffer[0]);
    }

    println!("Analysis found {} moves", moves.len());

    let best_move = find_best_development(game_state, moves);

    game_state.perform_move(&best_move).print();

    println!(
        "{} selected from {} valid moves.",
        best_move.repr(),
        analysis.valid_moves
    );
    println!(
        "Simulated {} moves, took {} seconds or {} ms",
        analysis.sim_moves,
        timer.elapsed().as_secs(),
        timer.elapsed().as_millis()
    );
    Some(best_move)
}

fn get_recursion_depth() -> u8 {
    loop {
        let res = input("Recursion depth: ").parse::<u8>();
        match res {
            Ok(depth) => return depth,
            Err(_) => {
                println!("Must be non-negative int");
                continue;
            }
        };
    }
}

fn main() {
    let mut game_state = parse_layout();

    if !GAME_LOOP {
        let depth = get_recursion_depth();
        loop {
            let mov_option = find_best_move(&game_state, depth);
            input("Press enter to continue");
            game_state = game_state.perform_move(&mov_option.unwrap());
        }
    }

    loop {
        let opponent_move = input_move("opponent", &game_state, true);
        game_state = game_state.perform_move(&opponent_move);

        let depth = get_recursion_depth();
        let mov_option = find_best_move(&game_state, depth);
        let engine_move = match mov_option {
            Some(mov) => {
                if input("Accept this move? y/n: ") == "n" {
                    input_move("engine", &game_state, false)
                } else {
                    mov
                }
            }
            None => input_move("engine", &game_state, false),
        };
        game_state = game_state.perform_move(&engine_move);
        for mov in game_state.get_possible_moves(true) {
            println!("{}", mov.repr());
        }
    }
}
