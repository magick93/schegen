use std::path::Path;

/// Shared test context containing paths and content
pub struct TestContext {
    pub model_path: String,
    pub endpoint_path: String,
    pub model_content: String,
    pub endpoint_content: String,
}

impl TestContext {
    pub fn new() -> Self {
        let model_path = "tests/generated/api/model.rs".to_string();
        let endpoint_path = "tests/generated/api/endpoint.rs".to_string();
        
        let model_content = std::fs::read_to_string(&model_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", model_path));
            
        let endpoint_content = std::fs::read_to_string(&endpoint_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", endpoint_path));

        Self {
            model_path,
            endpoint_path,
            model_content,
            endpoint_content,
        }
    }
}

/// Builder for creating CLI commands
pub struct CommandBuilder {
    args: Vec<&'static str>,
}

impl CommandBuilder {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn validate_openapi(mut self) -> Self {
        self.args.push("validate");
        self
    }

    pub fn process_dereference(mut self) -> Self {
        self.args.push("dereference");
        self
    }

    pub fn build(&self) -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("schegen")
            .args(&self.args);
        cmd
    }
}

/// Verify that a file exists and is not empty
pub fn assert_file_exists_and_not_empty(path: &str) {
    assert!(
        Path::new(path).exists(),
        "File {} should exist",
        path
    );
    
    let metadata = std::fs::metadata(path)
        .unwrap_or_else(|_| panic!("Failed to get metadata for {}", path));
        
    assert!(
        metadata.len() > 0,
        "File {} should not be empty",
        path
    );
}

/// Builder for model-specific assertions
pub struct ModelAssertions<'a> {
    content: &'a str,
}

impl<'a> ModelAssertions<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn has_derive(&self, derive: &str) -> &Self {
        assert!(
            self.content.contains(&format!("#[derive({})]", derive)),
            "Model should derive {}\nFound content:\n{}",
            derive,
            self.content
        );
        self
    }

    pub fn has_import(&self, import: &str) -> &Self {
        assert!(
            self.content.contains(&format!("use {};", import)),
            "Model should import {}\nFound content:\n{}",
            import,
            self.content
        );
        self
    }
}

/// Builder for endpoint-specific assertions
pub struct EndpointAssertions<'a> {
    content: &'a str,
}

impl<'a> EndpointAssertions<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn has_framework(&self, framework: &str) -> &Self {
        assert!(
            self.content.contains(&format!("{}::", framework)),
            "Endpoint should use {} framework\nFound content:\n{}",
            framework,
            self.content
        );
        self
    }

    pub fn has_error_handling(&self) -> &Self {
        assert!(
            self.content.contains("Error"),
            "Endpoint should include error handling\nFound content:\n{}",
            self.content
        );
        self
    }
}

/// Create model assertions builder
pub fn assert_model(content: &str) -> ModelAssertions {
    ModelAssertions::new(content)
}

/// Create endpoint assertions builder
pub fn assert_endpoint(content: &str) -> EndpointAssertions {
    EndpointAssertions::new(content)
}
