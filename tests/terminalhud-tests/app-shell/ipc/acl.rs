use super::fixture::Primary;
use std::{io, time::Duration};
use terminalhud_app::windows::{ipc, security::Identity};
use terminalhud_core::{Command, integration::Availability};
#[test]
fn anonymous_connection_is_denied_by_the_pipe_acl() {
    use windows_sys::Win32::{
        Security::{ImpersonateAnonymousToken, RevertToSelf},
        System::Threading::GetCurrentThread,
    };
    let mut primary = Primary::start();
    let identity = Identity::current().unwrap();
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            assert_ne!(unsafe { RevertToSelf() }, 0);
        }
    }
    assert_ne!(unsafe { ImpersonateAnonymousToken(GetCurrentThread()) }, 0);
    let restore = Restore;
    let denied = ipc::connect(&identity, primary.instance(), Duration::from_millis(100));
    drop(restore);
    assert!(denied.is_err());
    assert_eq!(
        denied.err().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        primary.call(Command::Status).status.availability,
        Availability::Absent
    );
    primary.call(Command::Quit);
    primary.wait();
}
