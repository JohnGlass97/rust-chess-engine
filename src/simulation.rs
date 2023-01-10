use std::{thread, time::Instant};

use crate::{gamestate::GameState, moves::Move, settings::THREADING, utils::AnalysisResult};

pub fn analyse(game_state: &GameState, depth: i8, root: bool) -> AnalysisResult {
    assert!(game_state.kings_alive && depth >= 0);

    if depth == 0 {
        return AnalysisResult {
            best_moves: Vec::new(),
            score: game_state.score,
            opponent_in_check: false,
            engine_no_moves: false,
            sim_moves: 0,
            tested_moves: 0,
        };
    }

    let engine_possible_moves: Vec<Move> = game_state.get_possible_moves(false);

    let total_outer = engine_possible_moves.len();
    let mut completed_outer: usize = 0;
    let timer = Instant::now();

    let mut sim_moves: u32 = 0;
    let mut tested_moves: u32 = 0;

    let mut best_moves: Vec<Move> = Vec::new();
    let mut best_score = i16::MIN;

    for engine_move in engine_possible_moves {
        let mut self_check = false;

        let game_state_1 = game_state.perform_move(&engine_move);
        sim_moves += 1;

        // If opponent king killable, immediately return
        if !game_state_1.kings_alive {
            return AnalysisResult {
                best_moves: Vec::from([engine_move]),
                score: game_state_1.score,
                opponent_in_check: true,
                engine_no_moves: false,
                sim_moves,
                tested_moves: 0,
            };
        }

        let opponent_possible_moves = game_state_1.get_possible_moves(true);

        let mut worst_case_score = i16::MAX;
        let mut found_valid_opponent_move = false;

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

            if THREADING && root {
                let handle = thread::spawn(move || -> AnalysisResult {
                    analyse(&game_state_2, depth - 1, false)
                });
                result_handles.push(handle);
            } else {
                results.push(analyse(&game_state_2, depth - 1, false));
            }
        }

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
                worst_case_score = -1000;
                break;
            }

            worst_case_score = i16::min(analysis.score, worst_case_score);
        }

        if self_check {
            continue;
        }

        tested_moves += 1;

        // Trapped opponent
        if !found_valid_opponent_move {
            let analysis = analyse(&game_state_1, 1, false);
            sim_moves += analysis.sim_moves;

            // If checkmate push with actual score, else -1000
            worst_case_score = if analysis.opponent_in_check {
                analysis.score
            } else {
                -1000
            };
        }

        if worst_case_score < best_score {
            continue;
        }
        if worst_case_score > best_score {
            best_moves.clear();
            best_score = worst_case_score
        }
        best_moves.push(engine_move);

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
    }

    let engine_no_moves = best_moves.len() == 0;

    AnalysisResult {
        best_moves,
        score: best_score,
        opponent_in_check: false,
        engine_no_moves,
        sim_moves,
        tested_moves,
    }
}
