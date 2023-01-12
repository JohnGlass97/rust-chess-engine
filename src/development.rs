use rand::Rng;

use crate::{
    gamestate::GameState,
    moves::{Move, MoveType},
    pieces::PieceClass,
    settings::{BOARD_WIDTH, RANDOM_FACTOR},
    simulation::analyse,
};

fn get_defended_score(
    defended_matrix: &[[f32; BOARD_WIDTH as usize]; BOARD_WIDTH as usize],
    mov: &Move,
) -> f32 {
    // Prefer moves to squares protected by multiple pieces
    let (dest, subtract) = match mov.move_type {
        MoveType::Standard(_, to, subtract) => (to, subtract),
        MoveType::DoubleAdvance(_, to) => (to, false),
        MoveType::EnPassant(_, to, _) => (to, true),
        MoveType::Castling(_) => return 0.,
        MoveType::Promotion(_, to, _, subtract) => (to, subtract),
        MoveType::Null => panic!("Engine provided move should not be null"),
    };

    let mut score = defended_matrix[dest.y as usize][dest.x as usize];
    if subtract {
        score -= 1.;
    }
    f32::max(score, 0.)
}

fn get_position_score(game_state: &GameState, mov: &Move) -> f32 {
    let (from, to) = match mov.move_type {
        MoveType::Standard(from, to, _) => (from, to),
        MoveType::DoubleAdvance(from, to) => (from, to),
        MoveType::EnPassant(from, to, _) => (from, to),
        MoveType::Castling(_) => return 0.,
        MoveType::Promotion(from, to, _, _) => (from, to),
        MoveType::Null => panic!("Engine provided move should not be null"),
    };

    let square = game_state.board[from.y as usize][from.x as usize];
    match square {
        Some(piece) => match piece.class {
            PieceClass::King => return if to.y == 0 { 1. } else { 0. },
            _ => (),
        },
        None => (),
    }

    let mut score = 0.;
    // Prefer moving up
    if from.y < 2 && to.y >= 2 {
        score += 2.;
        if to.y > 2 {
            score += 1.
        }
        // Prefer centre
        if to.x == 3 || to.x == 4 {
            score += 1.;
        }
    }

    score
}

fn get_double_move_score(new_state: &GameState) -> f32 {
    let result = analyse(new_state, 1, false);
    let score_delta = result.score_buffer[0] - new_state.score;
    i16::max(0, i16::min(5, score_delta)) as f32
}

fn get_opponent_trap_score(new_state: &GameState) -> f32 {
    let count = new_state.get_possible_moves(true).len() as i16;
    i16::max(0, 20 - count) as f32
}

pub fn find_best_development(game_state: &GameState, moves: Vec<Move>) -> Move {
    let mut rng = rand::thread_rng();

    let mut best_dev = f32::MIN;
    let mut best_move = Move {
        enemy: false,
        move_type: MoveType::Null,
    };

    let defended_matrix = game_state.get_defended_matrix();
    for mov in moves {
        let mut dev = 0.;
        let new_state = &game_state.perform_move(&mov);

        // Apply weightings to each component
        if game_state.score > 12 {
            let trap = get_opponent_trap_score(new_state);
            dev += trap * 1. / 30.;

            let double_move = get_double_move_score(new_state);
            dev += double_move * 1. / 15.;
        } else {
            let defended = get_defended_score(&defended_matrix, &mov);
            dev += f32::min(defended, 1.) * 1. / 3.;

            let position = get_position_score(game_state, &mov);
            dev += f32::min(position, 4.) * 1. / 6.;
        }

        assert!(dev < 1.1);

        dev *= 1. - RANDOM_FACTOR;
        dev += rng.gen::<f32>() * RANDOM_FACTOR;

        if dev > best_dev {
            best_move = mov;
            best_dev = dev;
        }
    }

    println!("Best dev: {}", best_dev);

    best_move
}
