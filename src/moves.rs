use crate::{
    pieces::{Board, Piece, PieceClass},
    settings::BOARD_WIDTH,
    utils::{pos_notation, CastlingState, SquareType, Vect},
};

const ROOK_VECTORS: [Vect; 4] = [
    Vect { x: 1, y: 0 },
    Vect { x: 0, y: 1 },
    Vect { x: -1, y: 0 },
    Vect { x: 0, y: -1 },
];

const BISHOP_VECTORS: [Vect; 4] = [
    Vect { x: 1, y: 1 },
    Vect { x: -1, y: 1 },
    Vect { x: -1, y: -1 },
    Vect { x: 1, y: -1 },
];

const KNIGHT_VECTORS: [Vect; 8] = [
    Vect { x: 2, y: 1 },
    Vect { x: -2, y: 1 },
    Vect { x: 2, y: -1 },
    Vect { x: -2, y: -1 },
    Vect { x: 1, y: 2 },
    Vect { x: 1, y: -2 },
    Vect { x: -1, y: 2 },
    Vect { x: -1, y: -2 },
];

pub enum MoveType {
    Standard(Vect, Vect),
    DoubleAdvance(Vect, Vect),
    EnPassant(Vect, Vect),
    Castling(bool), // True if queenside
    Promotion(Vect, Vect, Piece),
}

pub struct Move {
    pub enemy: bool,
    pub move_type: MoveType,
}

fn standard_move_notation(from: &Vect, to: &Vect) -> String {
    format!("{} -> {}", pos_notation(from), pos_notation(to))
}

impl Move {
    fn repr(&self) -> String {
        let mov = match &self.move_type {
            MoveType::Standard(from, to) => standard_move_notation(from, to),
            MoveType::DoubleAdvance(from, to) => standard_move_notation(from, to),
            MoveType::EnPassant(from, to) => standard_move_notation(from, to),
            MoveType::Castling(queenside) => String::from(if *queenside { "0-0-0" } else { "0-0" }),
            MoveType::Promotion(from, to, piece) => {
                format!("{} ({})", standard_move_notation(from, to), piece.repr())
            }
        };
        format!(
            "'{}: {}'",
            if self.enemy { "Opponent" } else { "Engine" },
            mov
        )
    }
}

fn check_squre(board: &Board, pos: &Vect) -> SquareType {
    let (x, y) = (pos.x, pos.y);
    if x >= BOARD_WIDTH || x < 0 {
        return SquareType::Invalid;
    };
    if y >= BOARD_WIDTH || y < 0 {
        return SquareType::Invalid;
    };

    let cell = &board[y as usize][x as usize];
    match cell {
        Some(piece) => {
            if piece.enemy {
                SquareType::Enemy
            } else {
                SquareType::Own
            }
        }
        None => SquareType::Free,
    }
}

impl Piece {
    fn get_linear_moves(
        &self,
        board: &Board,
        pos: Vect,
        vect_set: [Vect; 4],
        find_defended: bool,
    ) -> (Vec<Move>, Vec<Vect>) {
        let mut moves = Vec::new();
        let mut defended = Vec::new();

        let square_type = if self.enemy {
            SquareType::Enemy
        } else {
            SquareType::Own
        };

        for move_vect in vect_set {
            let mut square = pos.clone();
            loop {
                square.add(&move_vect);
                let state = check_squre(board, &square);
                if state == SquareType::Invalid {
                    break;
                }
                if find_defended {
                    defended.push(square.clone());
                }
                if state == square_type {
                    break;
                }
                moves.push(Move {
                    enemy: self.enemy,
                    move_type: MoveType::Standard(pos.clone(), square.clone()),
                });
                if state != SquareType::Free {
                    break;
                }
            }
        }
        (moves, defended)
    }
}

fn promotion_or_standard(piece: &Piece, from: Vect, to: Vect) -> Move {
    Move {
        enemy: piece.enemy,
        move_type: if to.y == 0 || to.y == BOARD_WIDTH - 1 {
            MoveType::Promotion(
                from,
                to,
                Piece {
                    // For now, the engine will only consider queening,
                    // but underpromotion is possible if entered manually
                    class: PieceClass::Queen,
                    enemy: piece.enemy,
                },
            )
        } else {
            MoveType::Standard(from, to)
        },
    }
}

fn pawn_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
    en_passant_target: &Vect,
) -> (Vec<Move>, Vec<Vect>) {
    let mut moves = Vec::new();
    let mut defended = Vec::new();

    // One forward
    let move_direction = if piece.enemy { -1 } else { 1 };
    let forwards_pos = Vect {
        x: pos.x,
        y: pos.y + move_direction,
    };

    {
        let state = check_squre(board, &forwards_pos);
        if state == SquareType::Free {
            moves.push(promotion_or_standard(piece, pos.clone(), forwards_pos));
        } else if state == SquareType::Invalid {
            println!("UNPROMOTED PAWN!");
            return (moves, defended);
        }
    }
    // Diagonal attack and en passant
    for x in [-1, 1] {
        let diagonal_pos = Vect {
            x: pos.x + x,
            y: pos.y + move_direction,
        };
        let state = check_squre(board, &diagonal_pos);
        if state == SquareType::Invalid {
            continue;
        }
        if find_defended {
            defended.push(diagonal_pos);
        }
        if state == SquareType::Enemy {
            moves.push(promotion_or_standard(piece, pos.clone(), diagonal_pos));
        } else if state != SquareType::Own && diagonal_pos.equals(en_passant_target) {
            moves.push(Move {
                enemy: piece.enemy,
                move_type: MoveType::EnPassant(pos.clone(), diagonal_pos),
            });
        }
    }

    // Double advance
    if pos.y == 1 || pos.y == BOARD_WIDTH - 2 {
        let forwards_pos = Vect {
            x: pos.x,
            y: pos.y + 2 * move_direction,
        };
        let state = check_squre(board, &forwards_pos);
        if state == SquareType::Free {
            moves.push(Move {
                enemy: piece.enemy,
                move_type: MoveType::DoubleAdvance(pos.clone(), forwards_pos),
            });
        }
    }

    (moves, defended)
}

fn knight_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
) -> (Vec<Move>, Vec<Vect>) {
    let mut moves = Vec::new();
    let mut defended = Vec::new();

    let square_type = if piece.enemy {
        SquareType::Enemy
    } else {
        SquareType::Own
    };
    for move_vect in KNIGHT_VECTORS {
        let mut square = pos.clone();
        square.add(&move_vect);
        let state = check_squre(board, &square);
        if state == SquareType::Invalid {
            continue;
        }
        if find_defended {
            defended.push(square.clone());
        }
        if state != square_type {
            moves.push(Move {
                enemy: piece.enemy,
                move_type: MoveType::Standard(pos.clone(), square),
            });
        }
    }

    (moves, defended)
}

fn bishop_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
) -> (Vec<Move>, Vec<Vect>) {
    let (moves, defended) = piece.get_linear_moves(board, pos, BISHOP_VECTORS, find_defended);
    (moves, defended)
}

fn rook_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
) -> (Vec<Move>, Vec<Vect>) {
    let (moves, defended) = piece.get_linear_moves(board, pos, ROOK_VECTORS, find_defended);
    (moves, defended)
}

fn queen_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
) -> (Vec<Move>, Vec<Vect>) {
    let (mut moves, mut defended) =
        piece.get_linear_moves(board, pos, BISHOP_VECTORS, find_defended);
    let (moves_2, defended_2) = piece.get_linear_moves(board, pos, ROOK_VECTORS, find_defended);
    moves.extend(moves_2);
    defended.extend(defended_2);
    (moves, defended)
}

fn king_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
    castling_state: CastlingState,
) -> (Vec<Move>, Vec<Vect>) {
    let mut moves = Vec::new();
    let mut defended = Vec::new();

    let square_type = if piece.enemy {
        SquareType::Enemy
    } else {
        SquareType::Own
    };

    // Move 1
    for vect_set in [BISHOP_VECTORS, ROOK_VECTORS] {
        for move_vect in vect_set {
            let mut square = pos.clone();
            square.add(&move_vect);
            let state = check_squre(board, &square);
            if state == SquareType::Invalid {
                continue;
            }
            if find_defended {
                defended.push(square.clone());
            }
            if state != square_type {
                moves.push(Move {
                    enemy: piece.enemy,
                    move_type: MoveType::Standard(pos.clone(), square),
                });
            }
        }
    }

    // Castling
    // True: Queenside, False: Kingside
    let mut castling_sides = Vec::new();

    if castling_state.queenside {
        castling_sides.push(true);
    }
    if castling_state.kingside {
        castling_sides.push(false);
    }
    for side in castling_sides {
        let move_vect = Vect {
            x: if side { -1 } else { 1 },
            y: 0,
        };
        let mut square = pos.clone();
        loop {
            square.add(&move_vect);
            let cell = &board[square.y as usize][square.x as usize];
            match cell {
                Some(rook) => match piece.class {
                    PieceClass::Rook => {
                        if rook.enemy == piece.enemy {
                            moves.push(Move {
                                enemy: piece.enemy,
                                move_type: MoveType::Castling(side),
                            });
                        }
                    }
                    _ => break,
                },
                None => (),
            }
        }
    }

    (moves, defended)
}
