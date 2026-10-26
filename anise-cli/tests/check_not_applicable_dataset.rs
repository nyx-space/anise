use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn check_not_applicable_dataset_returns_error_instead_of_panic() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "anise_cli_not_applicable_{}_{}.anise",
        std::process::id(),
        nonce
    ));

    // ANISE version header (0.10.0) followed by a dataset type of 0 (NotApplicable).
    std::fs::write(&path, [0x04, 0x03, 0x00, 0x0a, 0x00, 0x02, 0x01, 0x00])
        .expect("could not write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_anise-cli"))
        .args(["check", path.to_str().unwrap(), "0"])
        .output()
        .expect("failed to run anise-cli");

    let _ = std::fs::remove_file(&path);

    assert!(
        !output.status.success(),
        "a dataset of type NotApplicable should return an error"
    );
    assert_ne!(
        output.status.code(),
        Some(101),
        "check should not panic on a dataset of type NotApplicable"
    );

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        !combined.contains("panicked at"),
        "a dataset of type NotApplicable should be handled as an error, got output:\n{combined}"
    );
    assert!(
        combined.contains("malformed dataset type"),
        "expected the malformed dataset type error in output, got:\n{combined}"
    );
}
