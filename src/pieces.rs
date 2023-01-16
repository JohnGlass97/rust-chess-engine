use crate::{
    moves::{check_squre, Move, MoveType},
    settings::BOARD_WIDTH,
    utils::{CastlingPossibilities, SquareType, Vect},
};

// Max score attainable
pub const SCORE_RANGE: u64 = 2000;
pub const MAX_SCORE: i16 = 1000;

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

pub type Board = [[Option<Piece>; BOARD_WIDTH as usize]; BOARD_WIDTH as usize];

#[derive(Copy, Clone)]
pub enum PieceClass {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub class: PieceClass,
    pub enemy: bool,
}

impl Piece {
    pub fn repr(&self) -> char {
        let lower = match &self.class {
            PieceClass::Pawn => 'p',
            PieceClass::Bishop => 'b',
            PieceClass::Knight => 'n',
            PieceClass::Rook => 'r',
            PieceClass::Queen => 'q',
            PieceClass::King => 'k',
        };
        if self.enemy {
            lower.to_ascii_uppercase()
        } else {
            lower
        }
    }

    pub fn get_value(&self) -> i16 {
        match &self.class {
            PieceClass::Pawn => 1,
            PieceClass::Bishop => 3,
            PieceClass::Knight => 3,
            PieceClass::Rook => 5,
            PieceClass::Queen => 9,
            PieceClass::King => 500,
        }
    }

    pub fn all_moves(
        &self,
        board: &Board,
        pos: Vect,
        find_defended: bool,
        en_passant_midpoint: &Option<Vect>,
        castling: &CastlingPossibilities,
    ) -> (Vec<Move>, Vec<Vect>) {
        match &self.class {
            PieceClass::Pawn => pawn_moves(self, board, pos, find_defended, en_passant_midpoint),
            PieceClass::Bishop => bishop_moves(self, board, pos, find_defended),
            PieceClass::Knight => knight_moves(self, board, pos, find_defended),
            PieceClass::Rook => rook_moves(self, board, pos, find_defended, castling),
            PieceClass::Queen => queen_moves(self, board, pos, find_defended),
            PieceClass::King => king_moves(self, board, pos, find_defended),
        }
    }

    pub fn get_linear_moves(
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
                    move_type: MoveType::Standard(pos.clone(), square.clone(), true),
                });
                if state != SquareType::Free {
                    break;
                }
            }
        }
        (moves, defended)
    }
}

fn promotion_or_standard(piece: &Piece, from: Vect, to: Vect, self_defended: bool) -> Move {
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
                self_defended,
            )
        } else {
            MoveType::Standard(from, to, self_defended)
        },
    }
}

pub fn pawn_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
    en_passant_midpoint: &Option<Vect>,
) -> (Vec<Move>, Vec<Vect>) {
    let mut moves = Vec::new();
    let mut defended = Vec::new();

    let move_direction = if piece.enemy { -1 } else { 1 };

    let square_type = if piece.enemy {
        SquareType::Enemy
    } else {
        SquareType::Own
    };

    // One forward
    let mut one_forward = false;
    {
        let forwards_pos = Vect {
            x: pos.x,
            y: pos.y + move_direction,
        };
        let state = check_squre(board, &forwards_pos);
        if state == SquareType::Free {
            moves.push(promotion_or_standard(
                piece,
                pos.clone(),
                forwards_pos,
                false,
            ));
            one_forward = true;
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
        if state == SquareType::Free {
            if diagonal_pos.y == if piece.enemy { 2 } else { BOARD_WIDTH - 3 } {
                match en_passant_midpoint {
                    Some(midpoint) => {
                        if diagonal_pos.equals(midpoint) {
                            let en_passant_target = Vect {
                                x: pos.x + x,
                                y: pos.y,
                            };
                            moves.push(Move {
                                enemy: piece.enemy,
                                move_type: MoveType::EnPassant(
                                    pos.clone(),
                                    diagonal_pos,
                                    en_passant_target,
                                ),
                            });
                        }
                    }
                    None => (),
                }
            }
            continue;
        }
        if state != square_type {
            moves.push(promotion_or_standard(
                piece,
                pos.clone(),
                diagonal_pos,
                true,
            ));
        }
    }

    // Double advance
    if (pos.y == 1 || pos.y == BOARD_WIDTH - 2) && one_forward {
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

pub fn knight_moves(
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
                move_type: MoveType::Standard(pos.clone(), square, true),
            });
        }
    }

    (moves, defended)
}

pub fn bishop_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
) -> (Vec<Move>, Vec<Vect>) {
    let (moves, defended) = piece.get_linear_moves(board, pos, BISHOP_VECTORS, find_defended);
    (moves, defended)
}

pub fn rook_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
    castling: &CastlingPossibilities,
) -> (Vec<Move>, Vec<Vect>) {
    let (mut moves, defended) = piece.get_linear_moves(board, pos, ROOK_VECTORS, find_defended);

    // Castling
    if BOARD_WIDTH != 8 {
        return (moves, defended);
    }

    // True: Queenside, False: Kingside
    let queenside = pos.x == 0;

    // Only proceed if castling is possible
    if !castling.kingside && !queenside || !castling.queenside && queenside {
        return (moves, defended);
    }

    let move_vect = Vect {
        x: if queenside { 1 } else { -1 },
        y: 0,
    };
    let mut square = pos.clone();
    loop {
        // Check path to king is clear and that king is reached
        square.add(&move_vect);
        if check_squre(&board, &square) == SquareType::Invalid {
            // TODO: This shouldn't be reached if rook is present
            break;
        }
        let cell = &board[square.y as usize][square.x as usize];
        match cell {
            Some(king) => match king.class {
                PieceClass::King => {
                    if king.enemy == piece.enemy {
                        moves.push(Move {
                            enemy: piece.enemy,
                            move_type: MoveType::Castling(queenside),
                        });
                    }
                }
                _ => break,
            },
            None => (),
        }
    }

    (moves, defended)
}

pub fn queen_moves(
    piece: &Piece,
    board: &Board,
    pos: Vect,
    find_defended: bool,
) -> (Vec<Move>, Vec<Vect>) {
    let (mut moves, mut defended) =
        piece.get_linear_moves(board, pos, BISHOP_VECTORS, find_defended);
    let (mut moves_2, mut defended_2) =
        piece.get_linear_moves(board, pos, ROOK_VECTORS, find_defended);
    moves.append(&mut moves_2);
    defended.append(&mut defended_2);
    (moves, defended)
}

pub fn king_moves(
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

    // Move radius 1
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
                    move_type: MoveType::Standard(pos.clone(), square, true),
                });
            }
        }
    }

    (moves, defended)
}
