use crate::pieces::Board;

pub struct GameState {
    board: Board,
    score: i16,
    kings_alive: bool,
}
