//! Console probe for proving which native target was built and executed.
//! This is not the interactive test-window implementation.

fn main() {
    println!(
        "HudSwitch toolchain probe: os={} arch={} pid={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::process::id()
    );
}
