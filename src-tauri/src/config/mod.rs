// Configuration management module
// Handles both the global JSON config format and CasparCG XML config

mod caspar;
mod global;
mod schema;

pub use caspar::*;
pub use global::*;
pub use schema::*;
