use clap::Parser;
use miette::Result;
use std::path::PathBuf;

mod ast;
mod codegen;
mod errors;
mod lexer;
mod ownership;
mod parser;
mod types;

#[derive(Parser, Debug)]
#[command(name = "phprs")]
#[command(author, version, about = "PHP-Rust compiler with ownership semantics")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Compile a PHP file to native binary
    Compile {
        /// Input PHP file
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Emit LLVM IR instead of binary
        #[arg(long)]
        emit_llvm: bool,

        /// Enable debug output
        #[arg(long)]
        debug: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            emit_llvm,
            debug,
        } => {
            compile_file(&input, output.as_deref(), emit_llvm, debug)?;
        }
    }

    Ok(())
}

fn compile_file(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    emit_llvm: bool,
    debug: bool,
) -> Result<()> {
    use std::fs;

    let source =
        fs::read_to_string(input).map_err(|e| miette::miette!("Failed to read file: {}", e))?;

    if debug {
        eprintln!("=== Source ===\n{}", source);
    }

    // 1. Lexing
    let tokens = lexer::tokenize(&source)?;
    if debug {
        eprintln!("=== Tokens ===\n{:?}", tokens);
    }

    // 2. Parsing
    let ast = parser::parse(tokens)?;
    if debug {
        eprintln!("=== AST ===\n{:#?}", ast);
    }

    // 3. Type checking
    let typed_ast = types::check(&ast)?;
    if debug {
        eprintln!("=== Typed AST ===\n{:#?}", typed_ast);
    }

    // 4. Ownership checking
    ownership::check(&typed_ast)?;

    // 5. Code generation
    let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let mut path = input.to_path_buf();
        path.set_extension(if cfg!(windows) { "exe" } else { "" });
        path
    });

    codegen::compile(&typed_ast, &output_path, emit_llvm)?;

    println!("Compiled successfully: {}", output_path.display());
    Ok(())
}
