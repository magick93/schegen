use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

const PATH: &str = "schemas/ssvopenapi2.json";
const GENERATED_DIR: &str = "tests/generated";

mod test_utils {
    use super::*;
    
    /// Test context containing generated code and paths
    pub struct TestContext {
        pub endpoint_content: String,
        pub model_content: String,
        pub endpoint_path: String,
        pub model_path: String,
    }

    impl TestContext {
        /// Creates a new test context with generated code
        pub fn new() -> Self {
            let endpoint_path = format!("{}/api/endpoint.rs", GENERATED_DIR);
            let model_path = format!("{}/api/model.rs", GENERATED_DIR);
            
            let mut cmd = create_command();
            configure_codegen_command(&mut cmd).assert().success();
            
            let endpoint_content = fs::read_to_string(&endpoint_path).unwrap();
            let model_content = fs::read_to_string(&model_path).unwrap();
            
            Self {
                endpoint_content,
                model_content,
                endpoint_path,
                model_path,
            }
        }
    }

    /// Creates a new Command instance for the schegen binary
    pub fn create_command() -> Command {
        Command::cargo_bin("schegen").unwrap()
    }

    /// Configures a command for code generation with default parameters
    pub fn configure_codegen_command(cmd: &mut Command) -> &mut Command {
        cmd.arg("codegen")
            .arg("openapi")
            .arg(PATH)
            .arg("--template")
            .arg("templates/")
            .arg("--target-dir")
            .arg(GENERATED_DIR)
    }

    /// Asserts that generated code contains required axum features
    pub fn assert_axum_features(content: &str) {
        assert!(
            content.contains(".route(") || 
            content.contains(".with_state(") ||
            content.contains("Router::"),
            "Generated code should use axum's routing methods or router creation"
        );
        assert!(
            content.contains("async fn") ||
            content.contains("async move"),
            "Route handlers should be async functions"
        );
        assert!(
            content.contains("-> impl IntoResponse") || 
            content.contains("-> Result<") ||
            content.contains("-> Response"),
            "Route handlers should return IntoResponse, Result or Response"
        );
    }

    /// Asserts that generated code contains required utoipa features
    pub fn assert_utoipa_features(content: &str) {
        assert!(
            content.contains("utoipa::path") || 
            content.contains("utoipa::Path"),
            "Generated code should contain utoipa path annotations"
        );
        assert!(
            content.contains("utoipa::ToSchema") ||
            content.contains("utoipa::ToResponse"),
            "Generated code should contain utoipa schema definitions"
        );
        assert!(
            content.contains("operation_id") || 
            content.contains("operationId") ||
            content.contains("summary ="),
            "Generated code should contain operation IDs or summaries"
        );
        assert!(
            content.contains("responses(") ||
            content.contains("response ="),
            "Generated code should contain response definitions"
        );
    }

    /// Asserts that generated code contains proper error handling
    pub fn assert_error_handling(content: &str) {
        assert!(
            content.contains("IntoResponse"),
            "Generated code should implement IntoResponse for error handling"
        );
        assert!(
            content.contains("Json") || content.contains("json::"),
            "Generated code should use Json for response serialization"
        );
    }
}

use test_utils::*;

mod validation_tests {
    use super::*;

    #[test]
    fn test_validate_openapi() {
        let mut cmd = create_command();
        let assert = cmd.arg("validate").arg("openapi").arg(PATH).assert();
        assert.success().code(0);
    }

    #[test]
    fn test_process_dereference() {
        let mut cmd = create_command();
        let assert = cmd.arg("process").arg("dereference").arg(PATH).assert();
        assert.success().code(0);
    }
}

mod codegen_tests {
    use super::*;

    #[test]
    fn test_codegen() {
        let ctx = test_utils::TestContext::new();

        // Verify output files were created and are not empty
        assert!(Path::new(&ctx.model_path).exists());
        assert!(Path::new(&ctx.endpoint_path).exists());

        let models_metadata = fs::metadata(&ctx.model_path).unwrap();
        let endpoints_metadata = fs::metadata(&ctx.endpoint_path).unwrap();

        assert!(models_metadata.len() > 0, "models.rs should not be empty");
        assert!(endpoints_metadata.len() > 0, "endpoints.rs should not be empty");
    }
}

mod endpoint_tests {
    use super::*;

    #[test]
    fn test_axum_features() {
        let ctx = test_utils::TestContext::new();
        test_utils::assert_axum_features(&ctx.endpoint_content);
    }

    #[test]
    fn test_utoipa_features() {
        let ctx = test_utils::TestContext::new();
        test_utils::assert_utoipa_features(&ctx.endpoint_content);
    }

    #[test]
    fn test_error_handling() {
        let ctx = test_utils::TestContext::new();
        test_utils::assert_error_handling(&ctx.endpoint_content);
    }
}

mod model_tests {
    use super::*;

    #[test]
    fn test_response_derives() {
        let ctx = test_utils::TestContext::new();
        assert!(
            ctx.model_content.contains("utoipa::ToResponse"),
            "Model responses should derive ToResponse for response documentation"
        );
        assert!(
            ctx.model_content.contains("utoipa::ToSchema"),
            "Model responses should derive ToSchema for schema documentation"
        );
    }
}
