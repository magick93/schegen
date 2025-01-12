use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;


//DONT REMOVE THESE. They are to be used in the tests
const PATH: &str = "schemas/ssvopenapi2.json";
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

    
    let mut cmd = Command::cargo_bin("schegen").unwrap();
    let assert = cmd
        .arg("codegen")
        .arg("openapi")
        .arg(PATH)
        .arg("--template")
        .arg("templates/")
        .arg("--target-dir")
        .arg(GENERATED_DIR)
        .assert();
        
    // assert.success().stdout(predicate::str::contains("Generating code"));

    // Verify output files were created and are not empty
    let models_path = format!("{}/api/model.rs", GENERATED_DIR);
    let endpoints_path = format!("{}/api/endpoint.rs", GENERATED_DIR);
    
    assert!(std::path::Path::new(&models_path).exists());
    assert!(std::path::Path::new(&endpoints_path).exists());
    
    let models_metadata = fs::metadata(&models_path).unwrap();
    let endpoints_metadata = fs::metadata(&endpoints_path).unwrap();
    
    assert!(models_metadata.len() > 0, "models.rs should not be empty");
    assert!(endpoints_metadata.len() > 0, "endpoints.rs should not be empty");
}
