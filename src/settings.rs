// Changing this will break castling
pub const BOARD_WIDTH: i8 = 8;

// False: Engine on ranks 1 and 2, kings on e file
// True: Engine on ranks 7 and 8, kings on e file
pub const ENGINE_BLACK: bool = false;

// Use standard chess layout if false
// Dev mode disables castling
pub const DEV_MODE: bool = false;

// Layout for dev mode
// King must be on right, layout loaded
// flipped vertically only before loading
// into board
pub const LAYOUT: &str = "
R N B Q K B N R
P P P P P P P P
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
p p p p p p p p
r n b q k b n r
";
