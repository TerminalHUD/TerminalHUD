//! GUI process launch with an explicit inherited-handle allowlist.
mod attributes;
mod command_line;
mod handles;
mod launch;
mod process;
pub(crate) use launch::spawn;
pub(crate) use process::Process;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) static INHERITANCE_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
