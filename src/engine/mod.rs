#[allow(clippy::module_inception)]
mod engine;
pub use engine::Engine;

mod engine_repetition;
mod engine_uci;
mod search;

#[cfg(feature = "logging")]
mod engine_log;
#[cfg(feature = "logging")]
mod search_log;
