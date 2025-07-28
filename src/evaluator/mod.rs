#[allow(clippy::module_inception)]
mod evaluator;
pub use evaluator::{evaluate, game_phase, piece_square_value, piece_value};

mod piece_tables;
mod piece_values;
