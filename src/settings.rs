// Changing this will break castling
pub const BOARD_WIDTH: i8 = 8;

// False: Engine on ranks 1 and 2, kings on e file
// True: Engine on ranks 7 and 8, kings on e file
pub const ENGINE_BLACK: bool = false;

// Play a normal game
pub const GAME_LOOP: bool = true;

// All castling probabilities set to following
pub const CASTLING: bool = true;

pub const THREADING: bool = true;

pub const PRUNING: bool = false;

// Only affects development, not simulation
// Between 0 and 1
pub const RANDOM_FACTOR: f32 = 0.35;

// Standard layout, else one below
pub const STANDARD_BOARD: bool = true;

// King must be on right, layout loaded
// flipped vertically only before loading
// into memory
pub const LAYOUT: &str = r"
. . . . . . K .
. . . . . . . .
. r . . . . . .
. . . . . . . .
. r . . . . . .
. . . . . . . .
. . . . . . . .
. . . k . . . .
";
