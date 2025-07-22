#[allow(clippy::module_inception)]
mod evaluator;
pub use evaluator::{evaluate, piece_value};

mod piece_tables;
mod piece_values;
