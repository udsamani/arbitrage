use common::run_app;
use runner::ServerRunner;

mod adapters;
mod runner;
mod manager;
mod endpoint;
mod websocket;


fn main() {
    run_app(ServerRunner::default());
}
