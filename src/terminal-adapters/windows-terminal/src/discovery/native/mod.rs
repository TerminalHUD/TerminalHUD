mod folder;
mod identity;
mod packages;
use super::{candidate::candidate, types::*};

/// Query both current-user registered families. This reads metadata only and never selects a runtime.
pub fn discover_registered_terminals() -> RegisteredTerminalDiscovery {
    let local_app_data = folder::local_app_data();
    let channels = [TerminalChannel::Stable, TerminalChannel::Preview].map(|channel| {
        let candidates = packages::names(channel).map(|names| {
            names
                .into_iter()
                .map(|full_name| {
                    let identity = identity::identity(&full_name);
                    let path = packages::path(&full_name);
                    candidate(channel, full_name, identity, path, &local_app_data)
                })
                .collect()
        });
        RegisteredChannel {
            channel,
            candidates,
        }
    });
    RegisteredTerminalDiscovery {
        channels,
        local_app_data,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
