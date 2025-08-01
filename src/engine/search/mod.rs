#[allow(clippy::module_inception)]
mod search;
pub(super) use search::Search;

#[cfg(feature = "logging")]
mod log;
#[cfg(feature = "logging")]
pub(super) use log::{MoveTimeLimitKind, SearchLog};
