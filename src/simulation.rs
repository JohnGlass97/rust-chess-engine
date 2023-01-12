use std::{thread, time::Instant};

use crate::{
    gamestate::GameState,
    moves::Move,
    settings::THREADING,
    utils::{get_better_buffer, AnalysisResult, BetterBuffer},
};

pub fn analyse(game_state: &GameState, depth: u8, root: bool) -> AnalysisResult {
    assert!(game_state.kings_alive);

    if depth == 0 {
        return AnalysisResult {
            best_moves: None,
            score_buffer: Vec::from([game_state.score]),
            opponent_in_check: false,
            engine_no_moves: false,
            sim_moves: 0,
            valid_moves: 0,
        };
    }

    let engine_possible_moves: Vec<Move> = game_state.get_possible_moves(false);

    let total_outer = engine_possible_moves.len();
    let mut completed_outer: usize = 0;
    let timer = Instant::now();

    let mut sim_moves: u32 = 0;
    let mut valid_moves: u32 = 0;

    let mut best_moves: Vec<Move> = Vec::new();
    let mut best_score_buffer = Vec::from([i16::MIN]);

    for engine_move in engine_possible_moves {
        let mut self_check = false;

        let game_state_1 = game_state.perform_move(&engine_move);
        sim_moves += 1;

        // If opponent king killable, immediately return
        if !game_state_1.kings_alive {
            return AnalysisResult {
                best_moves: Some(Vec::from([engine_move])),
                score_buffer: Vec::from([game_state.score]),
                opponent_in_check: true,
                engine_no_moves: false,
                sim_moves,
                valid_moves: 0,
            };
        }

        let opponent_possible_moves = game_state_1.get_possible_moves(true);

        let mut result_handles: Vec<thread::JoinHandle<AnalysisResult>> = Vec::new();
        let mut results: Vec<AnalysisResult> = Vec::new();

        // Recursively analyse resulting gamestates for all moves
        for opponent_move in opponent_possible_moves {
            let game_state_2 = game_state_1.perform_move(&opponent_move);
            sim_moves += 1;

            if !game_state_2.kings_alive {
                self_check = true;
                break;
            }

            let func = move || -> AnalysisResult { analyse(&game_state_2, depth - 1, false) };

            if THREADING && root {
                let handle = thread::spawn(func);
                result_handles.push(handle);
            } else {
                results.push(func());
            }
        }

        let mut worst_case_buffer = Vec::from([i16::MAX]);
        let mut found_valid_opponent_move = false;

        for handle in result_handles {
            results.push(handle.join().unwrap());
        }

        for analysis in results {
            sim_moves += analysis.sim_moves;

            if analysis.opponent_in_check {
                // Opponent can't put self in check
                continue;
            }

            found_valid_opponent_move = true;

            if analysis.engine_no_moves {
                // Coule be checkmate or stalemate, reject both
                worst_case_buffer = Vec::from([-1000]);
                break;
            }

            match get_better_buffer(&worst_case_buffer, &analysis.score_buffer) {
                BetterBuffer::Left => {
                    worst_case_buffer = analysis.score_buffer;
                }
                _ => (),
            };
        }

        completed_outer += 1;
        if root {
            let fraction_done = completed_outer as f32 / total_outer as f32;
            let time_left = timer.elapsed().as_secs_f32() * (1.0 / fraction_done - 1.);
            println!(
                "{} / {}, {} secs left",
                completed_outer,
                total_outer,
                time_left.round()
            );
        }

        if self_check {
            continue;
        }

        valid_moves += 1;

        // Trapped opponent
        if !found_valid_opponent_move {
            let analysis = analyse(&game_state_1, 1, false);
            sim_moves += analysis.sim_moves;

            // If checkmate push with actual score, else -1000
            worst_case_buffer = if analysis.opponent_in_check {
                analysis.score_buffer
            } else {
                Vec::from([-1000])
            };
        }

        // Prioritising recurisve score followed by immediate score
        match get_better_buffer(&best_score_buffer, &worst_case_buffer) {
            BetterBuffer::Left => continue,
            BetterBuffer::Right => {
                best_moves.clear();
                best_score_buffer = worst_case_buffer;
            }
            BetterBuffer::Equal => (),
        }
        if root {
            best_moves.push(engine_move);
        }
    }

    let engine_no_moves = valid_moves == 0;

    assert!(!root || engine_no_moves || !best_moves.is_empty());

    best_score_buffer.push(game_state.score);

    AnalysisResult {
        best_moves: if root { Some(best_moves) } else { None },
        score_buffer: best_score_buffer,
        opponent_in_check: false,
        engine_no_moves,
        sim_moves,
        valid_moves,
    }
}
