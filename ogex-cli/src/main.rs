use clap::{Parser, Subcommand};
use colored::Colorize;
use ogex::{
    ConvertResult, Regex, TranspileResult, convert_all, transpile, transpile_debug,
    transpile_to_ogex, transpile_to_python,
};

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
    /// Convert regex syntax between flavors
    Convert {
        /// The pattern to convert (optional - shows help if not provided)
        pattern: Option<String>,
        /// Convert FROM legacy syntax TO Ogex syntax
        #[arg(long)]
        ogex: bool,
        /// Output as Python syntax: (?P<name>pattern)
        #[arg(long)]
        python: bool,
        /// Output as PCRE syntax: (?<name>pattern)
        #[arg(long)]
        pcre: bool,
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
        Commands::Convert {
            pattern,
            ogex,
            python,
            pcre,
            debug,
        } => cmd_convert(pattern.as_deref(), ogex, python, pcre, debug),
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

        // Always show groups if present
        if !m.groups.is_empty() || !m.named_groups.is_empty() {
            println!();
            println!("{}", "Capture groups:".bold());

            // Show numbered groups
            for (idx, (start, end)) in &m.groups {
                println!(
                    "  Group {}: {}..{} = {}",
                    idx,
                    start,
                    end,
                    &input[*start..*end].green()
                );
            }

            // Show named groups
            for (name, (start, end)) in &m.named_groups {
                println!(
                    "  Group ({}): {}..{} = {}",
                    name.cyan(),
                    start,
                    end,
                    &input[*start..*end].green()
                );
            }
        }

        // Verbose mode shows additional debug info
        if verbose {
            println!();
            println!("{}", "Debug info:".bold());
            println!("  Total numbered groups: {}", m.groups.len());
            println!("  Total named groups: {}", m.named_groups.len());
        }
    } else {
        println!("{}", "✗ No match".red());
    }
}

fn cmd_convert(pattern: Option<&str>, to_ogex: bool, to_python: bool, to_pcre: bool, debug: bool) {
    // Show help if no pattern provided
    let pattern = match pattern {
        Some(p) => p,
        None => {
            println!(
                "{}",
                "ogex convert - Convert regex syntax between flavors".bold()
            );
            println!();
            println!("Usage:");
            println!(
                "  ogex convert \"(name:pattern)\"           Convert to all flavors (default)"
            );
            println!("  ogex convert --ogex \"(?<name>p)\"       Convert legacy TO Ogex");
            println!("  ogex convert --python \"(?P<name>p)\"    Output as Python syntax");
            println!("  ogex convert --pcre \"(?<name>p)\"       Output as PCRE syntax");
            println!();
            println!("Flavors:");
            println!("  Ogex:   (name:pattern)     - Ogex native syntax");
            println!("  Python: (?P<name>pattern)  - Python re module syntax");
            println!("  PCRE:   (?<name>pattern)   - PCRE/.NET syntax");
            return;
        }
    };

    println!("{}", "Converting pattern...".bold());
    println!("  Input:  {}", pattern.cyan());
    println!();

    // Determine output mode
    let show_all = !to_ogex && !to_python && !to_pcre;

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
    } else if show_all {
        // Show all conversions
        match convert_all(pattern) {
            Ok(result) => {
                result.report();
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    } else {
        // Show specific conversion
        let result = if to_ogex {
            transpile_to_ogex(pattern)
        } else if to_python {
            transpile_to_python(pattern)
        } else {
            transpile(pattern) // default to PCRE
        };

        match result {
            Ok(output) => {
                let label = if to_ogex {
                    "Ogex"
                } else if to_python {
                    "Python"
                } else {
                    "PCRE"
                };
                println!("{}:", label.bold());
                println!("  {}", output.green());
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
