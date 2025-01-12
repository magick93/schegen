use assert_cmd::Command;
use predicates::prelude::*;

const PATH: &str = "schemas/ssvoapi.json";
const GENERATED_DIR: &str = "tests/generated";
#[test]
fn test_validate_openapi() {
    let mut cmd = Command::cargo_bin("schegen").unwrap();
    let assert = cmd
        .arg("validate")
        .arg("openapi")
        .arg(PATH)
        .assert();
        
    assert.success().stdout(predicate::str::contains("Validating openapi schema"));
}

#[test]
fn test_process_dereference() {
    let mut cmd = Command::cargo_bin("schegen").unwrap();
    let assert = cmd
        .arg("process")
        .arg("dereference")
        .arg(PATH)
        .assert();
        
    assert.success().stdout(predicate::str::contains("Processing dereference"));
}

#[test]
fn test_codegen() {
    let temp_dir = tempfile::tempdir().unwrap();
    let target_dir = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("schegen").unwrap();
    let assert = cmd
        .arg("codegen")
        .arg("openapi")
        .arg("schemas/ssvopenapi2.json")
        .arg("--template")
        .arg("templates/")
        .arg("--target-dir")
        .arg(GENERATED_DIR)
        .assert();
        
    assert.success().stdout(predicate::str::contains("Generating code"));
    
    // Verify output files were created
    assert!(std::path::Path::new(&format!("{}/models.rs", target_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/endpoints.rs", target_dir)).exists());
}
