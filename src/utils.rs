use std::io::{self, BufRead, BufReader, Write};

use crate::{
    moves::Move,
    settings::{BOARD_WIDTH, ENGINE_BLACK},
};

pub const LETTERS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

// Also represents vectors
#[derive(Copy, Clone)]
pub struct Vect {
    pub x: i8,
    pub y: i8,
}

impl Vect {
    pub fn add(&mut self, other: &Vect) {
        self.x += other.x;
        self.y += other.y;
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

#[derive(Copy, Clone)]
pub struct CastlingPossibilities {
    pub queenside: bool,
    pub kingside: bool,
}

pub struct AnalysisResult {
    pub best_moves: Option<Vec<Move>>,
    pub score_buffer: Vec<i16>,
    pub opponent_in_check: bool,
    pub engine_no_moves: bool,
    pub sim_moves: u32,
    pub valid_moves: u32,
}

pub fn pos_notation(pos: &Vect) -> String {
    let (x, mut y) = (pos.x, pos.y);
    if ENGINE_BLACK {
        // x = BOARD_WIDTH - x - 1;
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

pub enum BetterBuffer {
    Left,
    Right,
    Equal,
}

pub fn get_better_buffer(left: &Vec<i16>, right: &Vec<i16>) -> BetterBuffer {
    for (l, r) in left.iter().zip(right.iter()) {
        if l > r {
            return BetterBuffer::Left;
        }
        if l < r {
            return BetterBuffer::Right;
        }
    }

    // Shorter buffers are better
    let left_len = left.len();
    let right_len = right.len();
    if left_len > right_len {
        BetterBuffer::Right
    } else if left_len < right_len {
        BetterBuffer::Left
    } else {
        BetterBuffer::Equal
    }
}

pub fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Couldn't flush buffer");
    let result = BufReader::new(io::stdin()).lines().next().unwrap();
    result.unwrap()
}
