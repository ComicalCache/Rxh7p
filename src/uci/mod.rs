mod command;
pub use command::{GoCommandConfig, UciCommand};

mod receiver;
pub use receiver::{UciReceiver, UciSearchStop};

pub mod sender;
