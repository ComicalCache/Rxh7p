#[allow(clippy::module_inception)]
mod engine;
pub use engine::Engine;

mod repetition;
mod time_control;
mod uci;

#[cfg(feature = "logging")]
mod log;

mod search;
