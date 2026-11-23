use std::process::Command;
#[test]
fn help_keeps_protocol_stdout_empty() {
    let output = Command::new(env!("CARGO_BIN_EXE_systempulse-mcp"))
        .arg("--help")
        .output()
        .expect("MCP help starts");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("lecture seule"));
}
#[test]
fn unknown_option_is_rejected_without_protocol_pollution() {
    let output = Command::new(env!("CARGO_BIN_EXE_systempulse-mcp"))
        .arg("--execute=anything")
        .output()
        .expect("MCP starts");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
