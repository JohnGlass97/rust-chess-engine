# rust-chess-engine

A terminal based chess engine using the minimax algorithm and multithreading.
No opening databases have been utilised.

## Usage

Clone the project:

```
$ git clone https://github.com/JohnGlass97/rust-chess-engine.git
```

Install Rust and run the following:

```
$ cargo run --release
```

## State of the project

This chess engine was initally written in python, but was converted to rust and improvements were made.
This was done in about a week, so there are further improvments that could be made:

- Code documentation (this was written in about a week for my univeristy winter project, so this could be improved).
- Adding tests
- A full implemtation of Alpha-Beta pruning (it is only partially implemented currently).
- Better usage of multithreading (threads are spawned only at the root Minmax level,
  a library like `rayon` should be used instead of explicit thread spawning).
- Better heuristic measures for development of pieces.
- Use proper algebraic notation for inputting and outputting moves.
