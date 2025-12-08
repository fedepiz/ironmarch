mod simulation;
pub use simulation::Simulation;

mod object;
pub use object::{Object, ObjectId};

mod tick;
pub use tick::*;

mod view;
pub use view::*;

pub use spatial::geom::{Extents, V2};

mod aspects;
mod entities;
mod init;
mod names;
mod sites;
mod spawn;
