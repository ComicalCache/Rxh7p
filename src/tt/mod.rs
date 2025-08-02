#[allow(clippy::module_inception)]
mod tt;
pub use tt::TT;

mod entry;
pub use entry::{TtEntry, TtEntryFlag};
