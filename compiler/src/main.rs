use clap::Parser;
use miette::Result;
use std::path::PathBuf;

mod ast;
mod codegen;
mod errors;
mod lexer;
mod ownership;
mod parser;
mod resolver;
mod stdlib;
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

        /// Disable standard library (intrinsics)
        #[arg(long)]
        no_stdlib: bool,
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
            no_stdlib,
        } => {
            compile_file(&input, output.as_deref(), emit_llvm, debug, !no_stdlib)?;
        }
    }

    Ok(())
}

fn compile_file(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    emit_llvm: bool,
    debug: bool,
    use_stdlib: bool,
) -> Result<()> {
    use std::fs;

    let source =
        fs::read_to_string(input).map_err(|e| miette::miette!("Failed to read file: {}", e))?;

    if debug {
        eprintln!("=== Source ===\n{source}");
    }

    // 1. Lexing
    let tokens = lexer::tokenize(&source)?;
    if debug {
        eprintln!("=== Tokens ===\n{tokens:?}");
    }

    // 2. Parsing to compilation unit
    let unit = parser::parse_unit(tokens)?;
    if debug {
        eprintln!("=== Compilation Unit ===\n{unit:#?}");
    }

    // 3. Module resolution (if there are imports)
    let mut ast = if unit.uses.is_empty() && unit.namespace.is_none() {
        // Simple case: no namespace, no imports - just convert to Program
        ast::Program::from_unit(unit)
    } else {
        // Complex case: resolve imports
        let input_dir = input
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default();
        let mut resolver = resolver::ModuleResolver::new(vec![input_dir]);
        resolver.resolve(input.to_path_buf(), unit)?
    };

    // 3.5. Load stdlib intrinsics
    if use_stdlib {
        let stdlib_functions = stdlib::get_stdlib_functions();
        if debug {
            eprintln!(
                "=== Stdlib Functions: {} loaded ===",
                stdlib_functions.len()
            );
        }
        // Prepend stdlib functions (so user can override)
        let mut all_functions = stdlib_functions;
        all_functions.extend(ast.functions);
        ast.functions = all_functions;
    }

    if debug {
        eprintln!("=== AST ===\n{ast:#?}");
    }

    // 4. Type checking
    let typed_ast = types::check(&ast)?;
    if debug {
        eprintln!("=== Typed AST ===\n{typed_ast:#?}");
    }

    // 5. Ownership checking
    ownership::check(&typed_ast)?;

    // 6. Code generation
    let output_path = output.map_or_else(
        || {
            let mut path = input.to_path_buf();
            path.set_extension(if cfg!(windows) { "exe" } else { "" });
            path
        },
        std::path::Path::to_path_buf,
    );

    codegen::compile(&typed_ast, &output_path, emit_llvm)?;

    println!("Compiled successfully: {}", output_path.display());
    Ok(())
}
