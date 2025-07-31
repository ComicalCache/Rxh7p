#[allow(clippy::module_inception)]
mod search;
pub(super) use search::Search;

#[cfg(feature = "logging")]
mod log;
pub(super) use log::{MoveTimeLimitKind, SearchLog};
