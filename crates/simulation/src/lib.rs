mod simulation;
pub use simulation::Simulation;

mod entities;

mod object;
pub use object::{Object, ObjectId};

mod sites;

mod tick;
pub use tick::*;

mod view;
pub use view::*;

pub use spatial::geom::{Extents, V2};

mod init;
mod names;
mod spawn;
