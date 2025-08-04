#[allow(clippy::module_inception)]
mod evaluator;
pub use evaluator::{MATE_VALUE, evaluate, game_phase, piece_square_value, piece_value};

mod piece_tables;
mod piece_values;
