//! Pure scheduling: path overlap detection and wave assignment.

mod dag;
mod overlap;
mod waves;

pub use dag::{build_dag_plan, build_plan};
pub use overlap::{find_conflicts, paths_overlap};
pub use waves::build_execution_plan;
