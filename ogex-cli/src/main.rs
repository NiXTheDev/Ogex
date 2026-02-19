use clap::{Parser, Subcommand};
use colored::Colorize;
use ogex_core::{transpile, transpile_debug, Regex};

#[derive(Parser)]
#[command(name = "ogex")]
#[command(about = "Ogex - A custom regex engine with unified syntax")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a regex pattern against input
    Test {
        /// The regex pattern
        pattern: String,
        /// The input string to test
        input: String,
        /// Show detailed match information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Convert custom syntax to legacy regex syntax
    Convert {
        /// The pattern to convert
        pattern: String,
        /// Show AST debug output
        #[arg(short, long)]
        debug: bool,
    },
    /// Find all matches in input
    Find {
        /// The regex pattern
        pattern: String,
        /// The input string
        input: String,
    },
    /// Check if pattern matches
    Match {
        /// The regex pattern
        pattern: String,
        /// The input string
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            pattern,
            input,
            verbose,
        } => cmd_test(&pattern, &input, verbose),
        Commands::Convert { pattern, debug } => cmd_convert(&pattern, debug),
        Commands::Find { pattern, input } => cmd_find(&pattern, &input),
        Commands::Match { pattern, input } => cmd_match(&pattern, &input),
    }
}

fn cmd_test(pattern: &str, input: &str, verbose: bool) {
    println!("{}", "Testing pattern...".bold());
    println!("  Pattern: {}", pattern.cyan());
    println!("  Input:   {}", input.yellow());
    println!();

    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    if let Some(m) = regex.find(input) {
        println!("{}", "✓ Match found!".green().bold());
        println!("  Position: {}..{}", m.start, m.end);
        println!("  Match:    {}", m.as_str(input).green());

        if verbose && !m.groups.is_empty() {
            println!();
            println!("{}", "Capture groups:".bold());
            for (idx, (start, end)) in &m.groups {
                println!(
                    "  Group {}: {}..{} = {}",
                    idx,
                    start,
                    end,
                    &input[*start..*end].green()
                );
            }
        }
    } else {
        println!("{}", "✗ No match".red());
    }
}

fn cmd_convert(pattern: &str, debug: bool) {
    println!("{}", "Converting pattern...".bold());
    println!("  Input:  {}", pattern.cyan());
    println!();

    if debug {
        match transpile_debug(pattern) {
            Ok(result) => {
                result.report();
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    } else {
        match transpile(pattern) {
            Ok(result) => {
                println!("{}", "Output:".bold());
                println!("  {}", result.green());
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }
}

fn cmd_find(pattern: &str, input: &str) {
    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let matches = regex.find_all(input);

    if matches.is_empty() {
        println!("{}", "No matches found".red());
    } else {
        println!(
            "{} {}",
            "Found".bold(),
            format!("{} match(es)", matches.len()).green()
        );
        println!();

        for (i, m) in matches.iter().enumerate() {
            println!(
                "  [{}] {}..{} = {}",
                i + 1,
                m.start,
                m.end,
                m.as_str(input).green()
            );
        }
    }
}

fn cmd_match(pattern: &str, input: &str) {
    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    if regex.is_match(input) {
        println!("{}", "true".green());
        std::process::exit(0);
    } else {
        println!("{}", "false".red());
        std::process::exit(1);
    }
}
