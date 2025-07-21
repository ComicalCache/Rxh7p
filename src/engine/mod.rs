#[allow(clippy::module_inception)]
mod engine;
pub use engine::Engine;

mod engine_repetition;
mod engine_uci;
mod search;
