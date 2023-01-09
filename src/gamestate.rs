use crate::{
    moves::{Move, MoveType},
    pieces::{Board, Piece, PieceClass},
    settings::{BOARD_WIDTH, DEV_MODE, ENGINE_BLACK},
    utils::{CastlingPossibilities, Vect, LETTERS},
};

pub struct GameState {
    board: Board,
    score: i16,
    kings_alive: bool,
    engine_castling: CastlingPossibilities,
    opponent_castling: CastlingPossibilities,
    en_passant_midpoint: Option<Vect>,
}

fn standard_move(
    board: &mut Board,
    from: &Vect,
    to: &Vect,
    new_piece: Option<Piece>,
    castling: &mut CastlingPossibilities,
) -> (i16, bool) {
    let start_piece = board[from.y as usize][from.x as usize];
    let end_piece = board[to.y as usize][to.x as usize];
    board[from.y as usize][from.x as usize] = None;

    let mut king_killed = false;
    let mut score_delta = 0;
    match start_piece {
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
            _ => (),
        },
        None => panic!("No piece found at square {}, {}", from.x, from.y),
    }

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

    (score_delta, king_killed)
}

impl GameState {
    pub fn mov(&self, mov: &Move) -> GameState {
        let mut board = self.board.clone();
        let mut score = self.score;
        let mut kings_alive = self.kings_alive;
        let mut engine_castling = self.engine_castling;
        let mut opponent_castling = self.opponent_castling;
        let mut en_passant_midpoint = None;

        // Does this work?
        let castling = if mov.enemy {
            &mut opponent_castling
        } else {
            &mut engine_castling
        };

        match &mov.move_type {
            MoveType::Standard(from, to) => {
                let (score_delta, king_killed) =
                    standard_move(&mut board, from, to, None, castling);
                score += score_delta;
                kings_alive = !king_killed && kings_alive;
            }
            MoveType::DoubleAdvance(from, to) => {
                let (score_delta, king_killed) =
                    standard_move(&mut board, from, to, None, castling);
                score += score_delta;
                kings_alive = !king_killed && kings_alive;

                en_passant_midpoint = Some(Vect {
                    x: (from.x + to.x) / 2,
                    y: (from.y + to.y) / 2,
                });
            }
            MoveType::EnPassant(from, to, target) => {
                let (_, _) = standard_move(&mut board, from, to, None, castling);
                score += 1;

                board[target.y as usize][target.x as usize] = None
            }
            MoveType::Promotion(from, to, piece) => {
                let (score_delta, king_killed) =
                    standard_move(&mut board, from, to, Some(*piece), castling);
                score += score_delta;
                kings_alive = !king_killed && kings_alive;
            }
            MoveType::Castling(queenside) => {
                assert!(BOARD_WIDTH == 8);
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
                let (_, _) = standard_move(&mut board, &rook_start, &rook_end, None, castling);
                let (_, _) = standard_move(&mut board, &king_start, &king_end, None, castling);

                if mov.enemy {
                    opponent_castling.kingside = false;
                    opponent_castling.queenside = false;
                } else {
                    engine_castling.kingside = false;
                    engine_castling.queenside = false;
                }
            }
            MoveType::Null => (),
        }

        Self {
            board,
            score,
            kings_alive,
            engine_castling,
            opponent_castling,
            en_passant_midpoint,
        }
    }

    pub fn print(&self) {
        println!();
        for j in 0..BOARD_WIDTH {
            let mut out =
                String::from((if ENGINE_BLACK { j + 1 } else { BOARD_WIDTH - j }.to_string()));
            let row = self.board[(BOARD_WIDTH - 1 - j) as usize];
            for i in 0..BOARD_WIDTH {
                let square = row[if ENGINE_BLACK { BOARD_WIDTH - 1 - i } else { i } as usize];
                let char = match square {
                    Some(piece) => piece.repr(),
                    None => '.',
                };
                out += &format!(" {}", char);
            }
            println!("{}", out);
        }
        let mut out = String::from(" ");
        for i in 0..BOARD_WIDTH {
            out += &format!(
                " {}",
                LETTERS[if ENGINE_BLACK { BOARD_WIDTH - i - 1 } else { i } as usize]
            );
        }
        println!("{}", out);
        println!();
    }
}

pub fn parse_layout(_layout_string: &str) -> GameState {
    let mut board: Board = [[None; BOARD_WIDTH as usize]; BOARD_WIDTH as usize];
    let mut score = 0;

    let layout_string = _layout_string
        .strip_prefix("\n")
        .unwrap()
        .strip_suffix("\n")
        .unwrap();
    for (j, row) in layout_string.split("\n").enumerate() {
        let chars = row.chars();
        for i in 0..BOARD_WIDTH {
            let mut chars_iter = &mut chars.clone();
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
            queenside: !DEV_MODE,
            kingside: !DEV_MODE,
        },
        opponent_castling: CastlingPossibilities {
            queenside: !DEV_MODE,
            kingside: !DEV_MODE,
        },
        en_passant_midpoint: None,
    }
}
