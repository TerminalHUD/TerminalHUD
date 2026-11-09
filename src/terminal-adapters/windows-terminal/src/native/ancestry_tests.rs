use super::*;
fn identity(pid: u32) -> Result<ProcessIdentity, NativeError> {
    Ok(match pid {
        3 => ProcessIdentity::fixture(3, 300, "fixture.exe"),
        2 => ProcessIdentity::fixture(2, 200, "terminal.exe"),
        _ => ProcessIdentity::fixture(1, 400, "unrelated-reused.exe"),
    })
}
#[test]
fn owner_prefix_ignores_stale_unrelated_ancestor_above_exact_owner() {
    let parents = HashMap::from([(3, 2), (2, 1)]);
    let mut opened = Vec::new();
    let chain = walk(3, &parents, Some((2, Path::new("terminal.exe"))), |pid| {
        opened.push(pid);
        identity(pid)
    })
    .unwrap();
    assert_eq!(opened, [3, 2]);
    assert_eq!(
        chain.iter().map(|entry| entry.pid).collect::<Vec<_>>(),
        [3, 2]
    );
    assert!(matches!(
        walk(3, &parents, None, identity),
        Err(NativeError::AncestryReused {
            child_pid: 2,
            parent_pid: 1
        })
    ));
}
#[test]
fn stale_edge_before_owner_is_rejected_without_opening_higher_ancestors() {
    let parents = HashMap::from([(3, 2), (2, 1)]);
    let mut opened = Vec::new();
    let result = walk(3, &parents, Some((2, Path::new("terminal.exe"))), |pid| {
        opened.push(pid);
        Ok(ProcessIdentity::fixture(
            pid,
            if pid == 3 { 300 } else { 400 },
            if pid == 3 {
                "fixture.exe"
            } else {
                "terminal.exe"
            },
        ))
    });
    assert!(matches!(
        result,
        Err(NativeError::AncestryReused {
            child_pid: 3,
            parent_pid: 2
        })
    ));
    assert_eq!(opened, [3, 2]);
}
#[test]
fn wrong_executable_wrong_pid_and_absent_owner_do_not_form_an_anchor() {
    let parents = HashMap::from([(3, 2)]);
    for (pid, exe) in [(2, "other.exe"), (3, "terminal.exe"), (99, "terminal.exe")] {
        assert!(matches!(
            walk(3, &parents, Some((pid, Path::new(exe))), identity),
            Err(NativeError::Invalid(_))
        ));
    }
}
