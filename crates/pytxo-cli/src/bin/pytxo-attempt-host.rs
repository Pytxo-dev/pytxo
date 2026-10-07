//! Trusted entry point; all protocol and process logic lives in runner.
fn main() {
    std::process::exit(pytxo_runner::owned_launch::attempt_host_main());
}
