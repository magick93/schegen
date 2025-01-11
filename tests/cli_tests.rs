use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const PATH: &str = "schemas/ssvoapi.json";
#[test]
fn test_validate_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("schegen")?;
    
    let assert = cmd
        .arg("validate")
        .arg("openapi")
        .arg(PATH)
        .assert();
        
    assert.success();
    Ok(())
}

#[test]
fn test_process_dereference_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("schegen")?;
    
    let assert = cmd
        .arg("process")
        .arg("dereference")
        .arg(PATH)
        .assert();
        
    assert.success();
    Ok(())
}

#[test]
fn test_codegen_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("schegen")?;
    
    let assert = cmd
        .arg("codegen")
        .arg("openapi")
        .arg(PATH)
        .arg("--template")
        .arg("templates/")
        .arg("--target-dir")
        .arg(output_path)
        .assert();
        
    assert.success();
    
    // Verify files were generated
    assert!(temp_dir.path().join("models.rs").exists());
    assert!(temp_dir.path().join("endpoints.rs").exists());
    
    Ok(())
}

#[test]
fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("schegen")?;
    
    let assert = cmd.arg("--help").assert();
    assert.success()
        .stdout(predicate::str::contains("Wrapper CLI for schematools"));
        
    Ok(())
}
