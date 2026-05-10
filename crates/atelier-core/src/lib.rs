pub mod codex;
pub mod doctor;
pub mod job;
pub mod people;
pub mod project;
pub mod thread;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
