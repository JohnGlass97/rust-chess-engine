use crate::{
    moves::{Move, MoveType},
    pieces::{Board, Piece, PieceClass},
    settings::{BOARD_WIDTH, CASTLING, ENGINE_BLACK, LAYOUT, STANDARD_BOARD},
    utils::{CastlingPossibilities, Vect, LETTERS},
};

pub struct GameState {
    pub board: Board,
    pub score: i16,
    pub kings_alive: bool,
    pub engine_castling: CastlingPossibilities,
    pub opponent_castling: CastlingPossibilities,
    pub en_passant_midpoint: Option<Vect>,
}

fn print_board(board: &Board, direct: bool) {
    println!();
    for j in 0..BOARD_WIDTH {
        let mut out = if direct {
            String::new()
        } else {
            String::from(if ENGINE_BLACK { j + 1 } else { BOARD_WIDTH - j }.to_string())
        };
        let row = board[(BOARD_WIDTH - 1 - j) as usize];
        for i in 0..BOARD_WIDTH {
            let square = row[if ENGINE_BLACK && !direct {
                BOARD_WIDTH - 1 - i
            } else {
                i
            } as usize];
            let char = match square {
                Some(piece) => piece.repr(),
                None => '.',
            };
            if !direct || i != 0 {
                out += " ";
            }
            out += &format!("{}", char);
        }
        println!("{}", out);
    }
    if !direct {
        let mut out = String::from(" ");
        for i in 0..BOARD_WIDTH {
            out += &format!(
                " {}",
                LETTERS[if ENGINE_BLACK { BOARD_WIDTH - i - 1 } else { i } as usize]
            );
        }
        println!("{}", out);
    }
    println!();
}

fn standard_move(
    board: &mut Board,
    from: &Vect,
    to: &Vect,
    new_piece: Option<Piece>,
    castling: &mut CastlingPossibilities,
) -> (i16, bool, bool) {
    let start_piece = board[from.y as usize][from.x as usize];
    let end_piece = board[to.y as usize][to.x as usize];
    board[from.y as usize][from.x as usize] = None;

    let mut king_killed = false;
    let mut score_delta = 0;

    let mut subtract = true;

    // Disable castling if piece moved
    match &start_piece {
        Some(piece) => match piece.class {
            PieceClass::King => {
                castling.kingside = false;
                castling.queenside = false;
            }
            PieceClass::Rook => {
                if from.x == 0 {
                    castling.queenside = false;
                } else {
                    castling.kingside = false;
                }
            }
            PieceClass::Pawn => match &end_piece {
                Some(_) => (),
                None => subtract = false,
            },
            _ => (),
        },
        None => {
            println!("NO PIECE AT {}, {}", from.x, from.y);
        }
    }

    // Find change in score and check if king was killed
    match end_piece {
        Some(piece) => {
            if piece.enemy {
                score_delta = piece.get_value();
            } else {
                score_delta = -piece.get_value();
            }

            match piece.class {
                PieceClass::King => king_killed = true,
                _ => (),
            }
        }
        None => (),
    };

    // Handle promotion
    let replacement_piece = match new_piece {
        Some(piece) => {
            if piece.enemy {
                score_delta += 1 - piece.get_value();
            } else {
                score_delta += piece.get_value() - 1;
            }
            Some(piece)
        }
        None => start_piece,
    };
    board[to.y as usize][to.x as usize] = replacement_piece;

    (score_delta, king_killed, subtract)
}

impl GameState {
    pub fn get_possible_moves(&self, enemy: bool) -> Vec<Move> {
        let mut moves = Vec::new();

        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_WIDTH {
                let square = self.board[y as usize][x as usize];
                match square {
                    Some(piece) => {
                        if piece.enemy == enemy {
                            // Ignore defended positions and pass castling possibilities for given player
                            let (mut new_moves, _) = piece.all_moves(
                                &self.board,
                                Vect { x, y },
                                false,
                                &self.en_passant_midpoint,
                                if enemy {
                                    &self.opponent_castling
                                } else {
                                    &self.engine_castling
                                },
                            );
                            moves.append(&mut new_moves);
                        }
                    }
                    None => (),
                }
            }
        }
        moves
    }

    pub fn get_defended_matrix(&self) -> [[f32; BOARD_WIDTH as usize]; BOARD_WIDTH as usize] {
        let mut defended: Vec<Vect> = Vec::new();
        let enemy = false;

        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_WIDTH {
                let square = self.board[y as usize][x as usize];
                match square {
                    Some(piece) => {
                        if piece.enemy == enemy {
                            // Ignore defended positions and pass castling possibilities for given player
                            let (_, mut new_defended) = piece.all_moves(
                                &self.board,
                                Vect { x, y },
                                true,
                                &self.en_passant_midpoint,
                                if enemy {
                                    &self.opponent_castling
                                } else {
                                    &self.engine_castling
                                },
                            );
                            defended.append(&mut new_defended);
                        }
                    }
                    None => (),
                }
            }
        }

        let mut matrix = [[0.; BOARD_WIDTH as usize]; BOARD_WIDTH as usize];

        for pos in defended {
            matrix[pos.y as usize][pos.x as usize] += 1.;
        }

        matrix
    }

    pub fn perform_move_subtract(&self, mov: &Move) -> (GameState, bool) {
        // Subtract false if move is a pawn not attacking
        let mut board = self.board.clone();
        let mut score = self.score;
        let mut kings_alive = self.kings_alive;
        let mut engine_castling = self.engine_castling;
        let mut opponent_castling = self.opponent_castling;
        let mut en_passant_midpoint = None;

        // Get reference to castling possiblities for given player
        let castling = if mov.enemy {
            &mut opponent_castling
        } else {
            &mut engine_castling
        };

        let subtract;

        match &mov.move_type {
            MoveType::Standard(from, to, _) => {
                let (score_delta, king_killed, sub) =
                    standard_move(&mut board, from, to, None, castling);
                score += score_delta;
                kings_alive = !king_killed && kings_alive;
                subtract = sub;
            }
            MoveType::DoubleAdvance(from, to) => {
                let (score_delta, king_killed, _) =
                    standard_move(&mut board, from, to, None, castling);
                score += score_delta;
                kings_alive = !king_killed && kings_alive;

                // Update en passant destination square
                en_passant_midpoint = Some(Vect {
                    x: (from.x + to.x) / 2,
                    y: (from.y + to.y) / 2,
                });

                subtract = false;
            }
            MoveType::EnPassant(from, to, target) => {
                // Simply move piece, score delta will always be 0 and kings won't be killed
                standard_move(&mut board, from, to, None, castling);
                score += 1;
                board[target.y as usize][target.x as usize] = None;

                subtract = true;
            }
            MoveType::Promotion(from, to, piece, _) => {
                let (score_delta, king_killed, sub) =
                    standard_move(&mut board, from, to, Some(*piece), castling);
                score += score_delta;
                kings_alive = !king_killed && kings_alive;
                subtract = sub;
            }
            MoveType::Castling(queenside) => {
                assert!(BOARD_WIDTH == 8, "Board with must be 8 for castling");
                let y = if mov.enemy { 7 } else { 0 };
                let rook_start = Vect {
                    x: if *queenside { 0 } else { 7 },
                    y,
                };
                let rook_end = Vect {
                    x: if *queenside { 3 } else { 5 },
                    y,
                };
                let king_start = Vect { x: 4, y };
                let king_end = Vect {
                    x: if *queenside { 2 } else { 6 },
                    y,
                };
                standard_move(&mut board, &rook_start, &rook_end, None, castling);
                standard_move(&mut board, &king_start, &king_end, None, castling);

                if mov.enemy {
                    opponent_castling.kingside = false;
                    opponent_castling.queenside = false;
                } else {
                    engine_castling.kingside = false;
                    engine_castling.queenside = false;
                }

                subtract = false
            }
            MoveType::Null => {
                subtract = true;
            }
        }

        (
            Self {
                board,
                score,
                kings_alive,
                engine_castling,
                opponent_castling,
                en_passant_midpoint,
            },
            subtract,
        )
    }

    pub fn perform_move(&self, mov: &Move) -> GameState {
        let (game_state, _) = self.perform_move_subtract(mov);
        game_state
    }

    pub fn print(&self) {
        print_board(&self.board, false);
    }

    pub fn print_direct(&self) {
        print_board(&self.board, true);
    }
}

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

pub fn parse_layout() -> GameState {
    let mut board: Board = [[None; BOARD_WIDTH as usize]; BOARD_WIDTH as usize];
    let mut score = 0;

    let _layout_string = if STANDARD_BOARD {
        DEFAULT_LAYOUT
    } else {
        LAYOUT
    };

    let layout_string = _layout_string
        .strip_prefix("\n")
        .unwrap()
        .strip_suffix("\n")
        .unwrap();
    for (j, row) in layout_string.split("\n").enumerate() {
        let chars = row.chars();
        for i in 0..BOARD_WIDTH {
            let chars_iter = &mut chars.clone();
            let char = chars_iter.nth(2 * i as usize).unwrap();

            let square = if char == '.' {
                None
            } else {
                let enemy = char.is_ascii_uppercase();
                let class = match char.to_ascii_lowercase() {
                    'p' => PieceClass::Pawn,
                    'b' => PieceClass::Bishop,
                    'n' => PieceClass::Knight,
                    'r' => PieceClass::Rook,
                    'q' => PieceClass::Queen,
                    'k' => PieceClass::King,
                    _ => panic!("'{}' is not a valid piece", char),
                };
                let piece = Piece { enemy, class };
                if enemy {
                    score -= piece.get_value();
                } else {
                    score += piece.get_value();
                }
                Some(piece)
            };
            board[BOARD_WIDTH as usize - 1 - j][i as usize] = square;
        }
    }
    GameState {
        board,
        score,
        kings_alive: true,
        engine_castling: CastlingPossibilities {
            queenside: CASTLING,
            kingside: CASTLING,
        },
        opponent_castling: CastlingPossibilities {
            queenside: CASTLING,
            kingside: CASTLING,
        },
        en_passant_midpoint: None,
    }
}
