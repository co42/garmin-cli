mod event;
mod labels;
mod phase;
mod plan;
mod projection;
mod task;
mod workout;

// `TrainingPhase` and `CoachTask` are only embedded in `CoachPlan`; the
// re-exports keep them addressable as `garmin::TrainingPhase` etc. for
// consumers (and for consistency with the other submodule trees).
#[allow(unused_imports)]
pub use phase::*;
#[allow(unused_imports)]
pub use task::*;

pub use event::*;
pub use plan::*;
pub use projection::*;
pub use workout::*;
