use crate::{
    moves::{bishop_moves, king_moves, knight_moves, pawn_moves, queen_moves, rook_moves, Move},
    settings::BOARD_WIDTH,
    utils::{CastlingPossibilities, Vect},
};

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
}
