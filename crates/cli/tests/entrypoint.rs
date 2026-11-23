use std::process::Command;
#[test]
fn help_exits_without_collecting() {
    let output = Command::new(env!("CARGO_BIN_EXE_systempulse"))
        .arg("--help")
        .output()
        .expect("CLI");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("lecture seule"));
    assert!(output.stderr.is_empty());
}
#[test]
fn malicious_or_unbounded_arguments_are_rejected() {
    for args in [
        vec!["--count", "0"],
        vec!["--count", "65535"],
        vec!["--profile", "../../etc/passwd"],
        vec!["--execute", "rm"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_systempulse"))
            .args(args)
            .output()
            .expect("CLI");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
}
