use crate::settings::{BOARD_WIDTH, REVERSE_ORIGIN};

const LETTERS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

// Also represents vectors
#[derive(Copy, Clone)]
pub struct Vect {
    pub x: i8,
    pub y: i8,
}

impl Vect {
    pub fn add(&mut self, other: &Vect) {
        self.x += other.x;
        self.y == other.y;
    }
    pub fn equals(&self, other: &Vect) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(PartialEq, Eq)]
pub enum SquareType {
    Free,
    Own,
    Enemy,
    Invalid,
}

pub struct CastlingState {
    pub queenside: bool,
    pub kingside: bool,
}

pub fn pos_notation(pos: &Vect) -> String {
    let (mut x, mut y) = (pos.x, pos.y);
    if REVERSE_ORIGIN {
        x = BOARD_WIDTH - x - 1;
        y = BOARD_WIDTH - y - 1;
    }
    let x_usize = x as usize;
    let letter: char = if x_usize < LETTERS.len() {
        LETTERS[x_usize]
    } else {
        'X'
    };
    y += 1;
    format!("{letter}{y}")
}