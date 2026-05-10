pub mod codex;
pub mod codex_app_server;
pub mod codex_native;
pub mod doctor;
pub mod gateway;
pub mod job;
pub mod people;
pub mod project;
pub mod registry;
pub mod thread;
pub mod thread_events;
pub mod thread_interaction;
pub mod thread_queue;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
