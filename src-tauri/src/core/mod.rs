pub mod git;
pub mod launcher;
pub mod project;
pub mod scanner;
pub mod tag;

pub use project::{DiscoveredProject, Project};
pub use tag::{Tag, TagWithCount};

pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
