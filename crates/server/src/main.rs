use common::run_app;
use runner::ServerRunner;

mod adapters;
mod runner;
mod manager;


fn main() {
    run_app(ServerRunner::default());
}
