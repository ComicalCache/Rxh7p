#[allow(clippy::module_inception)]
mod tt;
pub use tt::TT;

mod tt_entry;
pub use tt_entry::{TtEntry, TtEntryFlag};
