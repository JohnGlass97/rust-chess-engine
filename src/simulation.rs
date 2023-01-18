use std::{thread, time::Instant};

use crate::{
    gamestate::GameState,
    moves::Move,
    pieces::{SCORE_BOUND, SCORE_RANGE},
    settings::THREADING,
};

pub struct AnalysisResult {
    pub best_moves: Option<Vec<Move>>,
    pub score_buffer: u64,
    pub end_score: i16,
    pub opponent_in_check: bool,
    pub engine_no_moves: bool,
    pub sim_moves: u32,
    pub valid_moves: u32,
}

pub fn analyse(game_state: &GameState, depth: u8, root: bool) -> AnalysisResult {
    assert!(game_state.kings_alive);

    if depth == 0 {
        return AnalysisResult {
            best_moves: None,
            score_buffer: game_state.get_normalized(),
            end_score: game_state.score,
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
    let mut best_score_buffer = 0;

    let mut end_score = 0;

    for engine_move in engine_possible_moves {
        let mut self_check = false;

        let game_state_1 = game_state.perform_move(&engine_move);
        sim_moves += 1;

        // If opponent king killable, immediately return
        if !game_state_1.kings_alive {
            assert!(!root, "Enemy king is in check");

            return AnalysisResult {
                best_moves: None,
                score_buffer: game_state_1.get_normalized(),
                end_score: game_state_1.score,
                opponent_in_check: true,
                engine_no_moves: false,
                sim_moves,
                valid_moves: 0,
            };
        }

        let opponent_possible_moves = game_state_1.get_possible_moves(true);

        let mut result_handles: Vec<thread::JoinHandle<AnalysisResult>> = Vec::new();

        let mut worst_case_buffer = u64::MAX;
        let mut cor_end_score = 0;
        let mut found_valid_opponent_move = false;

        // Recursively analyse resulting gamestates for all moves
        for opponent_move in opponent_possible_moves {
            let game_state_2 = game_state_1.perform_move(&opponent_move);
            sim_moves += 1;

            if !game_state_2.kings_alive {
                self_check = true;
                break;
            }

            if THREADING && root {
                let func = move || -> AnalysisResult { analyse(&game_state_2, depth - 1, false) };
                let handle = thread::spawn(func);
                result_handles.push(handle);
            } else {
                let analysis = analyse(&game_state_2, depth - 1, false);
                sim_moves += analysis.sim_moves;

                if analysis.opponent_in_check {
                    // Opponent can't put self in check
                    continue;
                }

                found_valid_opponent_move = true;

                if analysis.engine_no_moves {
                    // Coule be checkmate or stalemate, reject both
                    worst_case_buffer = 0;
                    break;
                }

                if analysis.score_buffer < worst_case_buffer {
                    worst_case_buffer = analysis.score_buffer;
                    cor_end_score = analysis.end_score;
                }

                // Prune remaining branches
                if worst_case_buffer < best_score_buffer {
                    break;
                }
            }
        }

        // The following is similar to above block, violating DRY,
        // but performance may be limited by using closures

        // Handle threaded analysis
        for handle in result_handles {
            let analysis = handle.join().unwrap();

            sim_moves += analysis.sim_moves;

            if analysis.opponent_in_check {
                // Opponent can't put self in check
                continue;
            }

            found_valid_opponent_move = true;

            if analysis.engine_no_moves {
                // Coule be checkmate or stalemate, reject both
                worst_case_buffer = 0;
                break;
            }

            if analysis.score_buffer < worst_case_buffer {
                worst_case_buffer = analysis.score_buffer;
                cor_end_score = analysis.end_score;
            }
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

            // If checkmate push with actual score, else worst possible
            worst_case_buffer = if analysis.opponent_in_check {
                // Fill buffer with score to represent depth it is acheived at
                SCORE_RANGE.pow(depth as u32) - 1
            } else {
                0
            };
            cor_end_score = analysis.end_score;
        }

        // Prioritising recurisve score followed by immediate score
        if worst_case_buffer < best_score_buffer {
            continue;
        }
        if worst_case_buffer > best_score_buffer {
            best_moves.clear();
            best_score_buffer = worst_case_buffer;
            end_score = cor_end_score;
        }
        if root {
            best_moves.push(engine_move);
        }
    }

    let engine_no_moves = valid_moves == 0;

    assert!(!root || engine_no_moves || !best_moves.is_empty());

    best_score_buffer = best_score_buffer * SCORE_RANGE + game_state.get_normalized();

    AnalysisResult {
        best_moves: if root { Some(best_moves) } else { None },
        score_buffer: best_score_buffer,
        end_score,
        opponent_in_check: false,
        engine_no_moves,
        sim_moves,
        valid_moves,
    }
}
