use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn convert_fk_missing_file_returns_error_instead_of_panic() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let missing_fk = format!("__missing_fk_{}_{}.txt", std::process::id(), nonce);
    let output_path = format!("./target/tmp/out_{}.epa", nonce);

    let output = Command::new(env!("CARGO_BIN_EXE_anise-cli"))
        .args(["convert-fk", &missing_fk, &output_path])
        .output()
        .expect("failed to run anise-cli");

    assert!(
        !output.status.success(),
        "missing FK input should return an error"
    );
    assert_ne!(
        output.status.code(),
        Some(101),
        "convert-fk should not panic on missing FK file"
    );

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        !combined.contains("panicked at"),
        "missing FK input should be handled as an error, got output:\n{combined}"
    );
    assert!(
        combined.contains("opening KPL file"),
        "expected IO error context in output, got:\n{combined}"
    );
}
