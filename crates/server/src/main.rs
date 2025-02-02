use common::run_app;
use runner::ServerRunner;

mod adapters;
mod runner;


fn main() {
    run_app(ServerRunner::default());
}
