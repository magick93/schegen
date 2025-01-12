use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "schegen")]
#[command(about = "Wrapper CLI for schematools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate OpenAPI or JSON schema
    Validate {
        /// Schema type (openapi or json)
        schema_type: String,
        /// Path to schema file
        file: String,
    },
    /// Process schema files
    Process {
        /// Operation to perform (name, dereference, merge-all-of, patch, merge-openapi, bump-openapi)
        operation: String,
        /// Path to schema file
        file: String,
        /// Output format (json or yaml)
        #[arg(short, long)]
        output: Option<String>,
        /// Path to output file
        #[arg(long)]
        to_file: Option<String>,
    },
    /// Generate code from schema
    Codegen {
        /// Schema type (openapi or json)
        schema_type: String,
        /// Path to schema file
        file: String,
        /// Template directory
        #[arg(long)]
        template: String,
        /// Target directory for generated code  
        #[arg(long)]
        target_dir: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { schema_type, file } => {
            let status = Command::new("schematools")
                .arg("validate")
                .arg(&schema_type)
                .arg(&file)
                
                .status()?;
            
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Commands::Process {
            operation,
            file,
            output,
            to_file,
        } => {
            let mut cmd = Command::new("schematools");
            cmd.arg("process").arg(&operation).arg(&file);

            if let Some(format) = output {
                cmd.arg("--output").arg(format);
            }

            if let Some(output_file) = to_file {
                cmd.arg("--to-file").arg(output_file);
            }

            let status = cmd.status()?;
            
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Commands::Codegen {
            schema_type,
            file,
            template,
            target_dir,
        } => {
            let status = Command::new("schematools")
                
                // .arg("-vvvv ")
                .arg("codegen")
                .arg(&schema_type)
                .arg(&file)
                .arg("--template")
                .arg(&template)
                .arg("--target-dir")
                .arg(&target_dir)
                .status()?;
            
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }

    Ok(())
}
