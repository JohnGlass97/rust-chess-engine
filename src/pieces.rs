use crate::{settings::BOARD_WIDTH, utils::Vect};

pub type Board = [[Option<Piece>; BOARD_WIDTH as usize]; BOARD_WIDTH as usize];

pub enum PieceClass {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

pub struct Piece {
    pub class: PieceClass,
    pub enemy: bool,
}

impl Piece {
    pub fn repr(&self) -> char {
        let lower = match &self.class {
            PieceClass::Pawn => ('p'),
            PieceClass::Bishop => ('b'),
            PieceClass::Knight => ('n'),
            PieceClass::Rook => ('r'),
            PieceClass::Queen => ('q'),
            PieceClass::King => ('k'),
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
            PieceClass::King => 200,
        }
    }
}
