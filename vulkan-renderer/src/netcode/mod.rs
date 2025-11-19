mod connection;
mod rollback;

pub use rollback::{InputHistory, PacketInputState, Rollback};

pub const MAX_ROLLBACK_FRAMES: usize = 10;
