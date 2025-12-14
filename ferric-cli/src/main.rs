//! Ferric CLI - Command-line tool for the Ferric framework
//!
//! Provides scaffolding, building, and development tools for Ferric applications.

use clap::{Parser, Subcommand};
use console::style;
use std::process::ExitCode;

mod cache_busting;
mod commands;
mod config;
mod scss;
mod server;
mod tailwind;
mod templates;
mod utils;
mod wasm_chunking;

#[derive(Parser)]
#[command(name = "ferric")]
#[command(author = "Ferric Contributors")]
#[command(version)]
#[command(about = "CLI tool for the Ferric framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Ferric project
    New {
        /// Name of the project
        name: String,

        /// Enable server-side rendering support
        #[arg(long)]
        ssr: bool,

        /// Skip git initialization
        #[arg(long)]
        skip_git: bool,

        /// Use a specific template
        #[arg(long, default_value = "default")]
        template: String,
    },

    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long, short)]
        release: bool,

        /// Target platform (browser, server, or universal)
        #[arg(long, default_value = "browser")]
        target: String,

        /// Watch for changes and rebuild
        #[arg(long, short)]
        watch: bool,
    },

    /// Start development server
    Serve {
        /// Port to serve on
        #[arg(long, short, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Open browser automatically
        #[arg(long)]
        open: bool,

        /// Enable SSR mode
        #[arg(long)]
        ssr: bool,
    },

    /// Generate a new component, service, or other artifact
    Generate {
        #[command(subcommand)]
        artifact: GenerateArtifact,
    },

    /// Run tests
    Test {
        /// Run in watch mode
        #[arg(long, short)]
        watch: bool,

        /// Run specific test
        #[arg(long)]
        filter: Option<String>,
    },

    /// Clean build artifacts
    Clean {
        /// Also remove node_modules
        #[arg(long)]
        all: bool,
    },

    /// Show project info
    Info,

    /// Add SSR support to an existing project
    AddSsr,

    /// Compile SCSS files to CSS
    Scss {
        /// Input SCSS file or directory
        input: String,

        /// Output CSS file or directory
        #[arg(long, short)]
        output: Option<String>,

        /// Watch for changes
        #[arg(long, short)]
        watch: bool,

        /// Minify output
        #[arg(long)]
        minify: bool,
    },

    /// Compile TailwindCSS using PostCSS
    Tailwind {
        /// Input CSS file (defaults to src/styles.css)
        #[arg(long, short)]
        input: Option<String>,

        /// Output CSS file (defaults to dist/styles.css)
        #[arg(long, short)]
        output: Option<String>,

        /// Watch for changes and recompile
        #[arg(long, short)]
        watch: bool,

        /// Minify output CSS
        #[arg(long)]
        minify: bool,

        /// Initialize TailwindCSS in the project
        #[arg(long)]
        init: bool,
    },
}

#[derive(Subcommand)]
enum GenerateArtifact {
    /// Generate a new component
    Component {
        /// Name of the component
        name: String,

        /// Generate inline template
        #[arg(long)]
        inline_template: bool,

        /// Generate inline styles
        #[arg(long)]
        inline_style: bool,
    },

    /// Generate a new service
    Service {
        /// Name of the service
        name: String,
    },

    /// Generate a new directive
    Directive {
        /// Name of the directive
        name: String,
    },

    /// Generate a new guard
    Guard {
        /// Name of the guard
        name: String,
    },

    /// Generate a new pipe
    Pipe {
        /// Name of the pipe
        name: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New {
            name,
            ssr,
            skip_git,
            template,
        } => commands::new::run(&name, ssr, skip_git, &template).await,

        Commands::Build {
            release,
            target,
            watch,
        } => commands::build::run(release, &target, watch).await,

        Commands::Serve {
            port,
            host,
            open,
            ssr,
        } => commands::serve::run(port, &host, open, ssr).await,

        Commands::Generate { artifact } => match artifact {
            GenerateArtifact::Component {
                name,
                inline_template,
                inline_style,
            } => commands::generate::component(&name, inline_template, inline_style).await,
            GenerateArtifact::Service { name } => commands::generate::service(&name).await,
            GenerateArtifact::Directive { name } => commands::generate::directive(&name).await,
            GenerateArtifact::Guard { name } => commands::generate::guard(&name).await,
            GenerateArtifact::Pipe { name } => commands::generate::pipe(&name).await,
        },

        Commands::Test { watch, filter } => commands::test::run(watch, filter).await,

        Commands::Clean { all } => commands::clean::run(all).await,

        Commands::Info => commands::info::run().await,

        Commands::AddSsr => commands::add_ssr::run().await,

        Commands::Scss {
            input,
            output,
            watch,
            minify,
        } => scss::compile_command(&input, output.as_deref(), watch, minify).await,

        Commands::Tailwind {
            input,
            output,
            watch,
            minify,
            init,
        } => {
            if init {
                match std::env::current_dir() {
                    Ok(cwd) => tailwind::init_command(&cwd).await,
                    Err(e) => Err(anyhow::anyhow!("Failed to get current directory: {}", e)),
                }
            } else {
                tailwind::compile_command(input.as_deref(), output.as_deref(), watch, minify).await
            }
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            ExitCode::FAILURE
        }
    }
}
