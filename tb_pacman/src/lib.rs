extern crate serde;
extern crate serde_json;
extern crate toybox_core;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate schemars;
extern crate rand;

// pub mod amidar;
pub mod pacman;
mod digit_sprites;
// mod types;
mod typespacman;



// pub use crate::types::Amidar as Pacman;
pub use crate::typespacman::Pacman;
pub use crate::typespacman::State;
