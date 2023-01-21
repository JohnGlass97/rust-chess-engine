use crate::{
    gamestate::GameState,
    pieces::{Board, Piece, PieceClass},
    settings::{BOARD_WIDTH, ENGINE_BLACK},
    utils::{input, pos_notation, SquareType, Vect, LETTERS},
};

pub enum MoveType {
    Standard(Vect, Vect, bool), // True if destination is self defended
    DoubleAdvance(Vect, Vect),
    EnPassant(Vect, Vect, Vect), // Last vect is en_passant_target (piece to remove)
    Castling(bool),              // True if queenside
    Promotion(Vect, Vect, Piece, bool), // True if destination is self defended
    Null,
}

pub struct Move {
    pub enemy: bool,
    pub move_type: MoveType,
}

fn standard_move_notation(from: &Vect, to: &Vect) -> String {
    format!("{} -> {}", pos_notation(from), pos_notation(to))
}

impl Move {
    pub fn repr(&self) -> String {
        let mov = match &self.move_type {
            MoveType::Standard(from, to, _) => standard_move_notation(from, to),
            MoveType::DoubleAdvance(from, to) => standard_move_notation(from, to),
            MoveType::EnPassant(from, to, _) => standard_move_notation(from, to),
            MoveType::Castling(queenside) => String::from(if *queenside { "0-0-0" } else { "0-0" }),
            MoveType::Promotion(from, to, piece, _) => {
                format!("{} ({})", standard_move_notation(from, to), piece.repr())
            }
            MoveType::Null => String::from("NULL"),
        };
        format!(
            "'{}: {}'",
            if self.enemy { "Opponent" } else { "Engine" },
            mov
        )
    }
}

pub fn check_squre(board: &Board, pos: &Vect) -> SquareType {
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

fn get_position(prompt: &str) -> Vect {
    loop {
        let inp = input(prompt);
        if inp.len() != 2 {
            println!("Position must be 2 characters");
            continue;
        }
        let chars = inp.as_bytes();
        let x_res = LETTERS.iter().position(|&r| r == chars[0] as char);
        let x = match x_res {
            Some(i) => i as i8,
            None => {
                println!("First character must be file");
                continue;
            }
        };
        let y_res = (chars[1] as char).to_string().parse::<i8>();
        let y = match y_res {
            Ok(j) => {
                if j > 0 && j <= BOARD_WIDTH {
                    if ENGINE_BLACK {
                        BOARD_WIDTH - j as i8
                    } else {
                        j - 1 as i8
                    }
                } else {
                    {
                        println!("Second character must be rank");
                        continue;
                    }
                }
            }
            Err(_) => {
                println!("Second character must be int");
                continue;
            }
        };
        return Vect { x, y };
    }
}

fn gen_move(choice: &str, from: Vect, to: Vect, subtract: bool, enemy: bool) -> Move {
    let move_type = match choice {
        "1" => MoveType::Standard(from, to, subtract),
        "2" => MoveType::DoubleAdvance(from, to),
        "3" => MoveType::EnPassant(from, to, Vect { x: from.y, y: to.x }),
        "5" => {
            let class = match input("Character of piece, defaults to queen: ").as_str() {
                "b" => PieceClass::Bishop,
                "n" => PieceClass::Knight,
                "r" => PieceClass::Rook,
                _ => PieceClass::Queen,
            };
            let piece = Piece { class, enemy };
            MoveType::Promotion(from, to, piece, subtract)
        }
        _ => panic!("Couldn't match move type"),
    };
    Move { enemy, move_type }
}

fn get_move(label: &str, game_state: &GameState, enemy: bool) -> Move {
    println!("\nSelect move type for {}: ", label);
    println!("1: Standard");
    println!("2: Double Advance");
    println!("3: En Passant");
    println!("4: Castling");
    println!("5: Promotion");
    println!("_: Null");
    let choice = input("> ");
    match choice.as_str() {
        "4" => {
            let queenside = input("Queenside or Kingside? q/k: ") == "q";
            return Move {
                move_type: MoveType::Castling(queenside),
                enemy,
            };
        }
        "1" | "2" | "3" | "5" => (),
        _ => {
            return Move {
                move_type: MoveType::Null,
                enemy,
            }
        }
    };

    let from = get_position("From: ");
    let to = get_position("To: ");

    let mut mov = gen_move(choice.as_str(), from, to, true, enemy);
    let (_, subtract) = game_state.perform_move_subtract(&mov);
    if !subtract {
        mov = gen_move(choice.as_str(), from, to, false, enemy);
    }
    mov
}

pub fn input_move(label: &str, game_state: &GameState, enemy: bool) -> Move {
    loop {
        let mov = get_move(label, game_state, enemy);
        game_state.perform_move(&mov).print();
        if input("Are you sure? y/n: ") == "y" {
            return mov;
        }
    }
}
