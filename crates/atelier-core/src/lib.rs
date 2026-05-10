pub mod codex;
pub mod codex_native;
pub mod doctor;
pub mod job;
pub mod people;
pub mod project;
pub mod registry;
pub mod thread;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
