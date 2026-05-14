use std::process::Command;

fn ogex() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ogex"))
}

#[test]
fn convert_emits_pcre_named_group() {
    let output = ogex()
        .args(["convert", "(name:abc)"])
        .output()
        .expect("spawn ogex convert");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("(?<name>abc)"),
        "unexpected stdout: {stdout}"
    );
}

#[test]
fn test_subcommand_reports_match() {
    let output = ogex()
        .args(["test", "(name:hello)", "hello world"])
        .output()
        .expect("spawn ogex test");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Match found"),
        "unexpected stdout: {stdout}"
    );
}

#[test]
fn match_subcommand_no_match_is_error() {
    let output = ogex()
        .args(["match", "a", "b"])
        .output()
        .expect("spawn ogex match");
    assert!(!output.status.success());
}
