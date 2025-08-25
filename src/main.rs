use clap::{Parser, Subcommand, ValueEnum};
use grid_calculator::{Config, GridCalculator, GridType, PositionMode, Strategy};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Calculate grid levels for trading
    Calculate {
        /// Initial price
        #[arg(short, long)]
        price: f64,

        /// Grid type
        #[arg(short, long, value_enum)]
        grid_type: GridTypeArg,

        /// Grid percentages (comma-separated, e.g., "1,2,3,5")
        #[arg(short = 'l', long)]
        levels: String,

        /// Position sizing mode
        #[arg(short = 'm', long, value_enum)]
        mode: PositionModeArg,

        /// Initial position size
        #[arg(short = 's', long, default_value = "100.0")]
        size: f64,

        /// Multiplier for position sizing (used in multiplier modes)
        #[arg(short = 'x', long, default_value = "1.0")]
        multiplier: f64,
    },
    
    /// Run calculation from a TOML config file
    FromConfig {
        /// Path to TOML configuration file
        #[arg(short, long)]
        config: PathBuf,
        
        /// Strategy name to use (optional, uses main config if not specified)
        #[arg(short, long)]
        strategy: Option<String>,
    },
    
    /// List all strategies in a config file
    ListStrategies {
        /// Path to TOML configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum GridTypeArg {
    /// Fixed price grid (relative to initial price)
    Fixed,
    /// Average price grid (relative to average price)
    Average,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PositionModeArg {
    /// Fixed size for each grid level
    Fixed,
    /// Multiple of current position
    CurrentMultiple,
    /// Multiple of last increment
    IncrementMultiple,
}

impl From<GridTypeArg> for GridType {
    fn from(arg: GridTypeArg) -> Self {
        match arg {
            GridTypeArg::Fixed => GridType::Fixed,
            GridTypeArg::Average => GridType::Average,
        }
    }
}

impl From<PositionModeArg> for PositionMode {
    fn from(arg: PositionModeArg) -> Self {
        match arg {
            PositionModeArg::Fixed => PositionMode::Fixed,
            PositionModeArg::CurrentMultiple => PositionMode::CurrentMultiple,
            PositionModeArg::IncrementMultiple => PositionMode::IncrementMultiple,
        }
    }
}

fn parse_grid_type(grid_type: &str) -> Result<GridType, String> {
    match grid_type {
        "fixed" => Ok(GridType::Fixed),
        "average" => Ok(GridType::Average),
        _ => Err(format!("Invalid grid type: {}", grid_type)),
    }
}

fn parse_position_mode(mode: &str) -> Result<PositionMode, String> {
    match mode {
        "fixed" => Ok(PositionMode::Fixed),
        "current-multiple" => Ok(PositionMode::CurrentMultiple),
        "increment-multiple" => Ok(PositionMode::IncrementMultiple),
        _ => Err(format!("Invalid position mode: {}", mode)),
    }
}

fn print_calculation_header(price: f64, grid_type: &str, mode: &str, size: f64, multiplier: Option<f64>) {
    println!("\n=== Grid Trading Calculator ===");
    println!("Initial Price: ${:.2}", price);
    println!("Grid Type: {}", grid_type);
    println!("Position Mode: {}", mode);
    println!("Base Size: {:.2}", size);
    if let Some(mult) = multiplier {
        if mode.contains("multiple") {
            println!("Multiplier: {:.2}x", mult);
        }
    }
    println!("\n{:-<60}", "");
    println!("{:<5} {:>10} {:>10} {:>10} {:>10} {:>10}", 
             "Grid", "Level %", "Price", "Size", "Total", "Avg Price");
    println!("{:-<60}", "");
}

fn print_calculation_results(calculator: &mut GridCalculator, levels: &[f64]) {
    for (i, &level) in levels.iter().enumerate() {
        let result = calculator.calculate_grid(level);
        println!("{:<5} {:>10.2}% {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
                 i + 1,
                 level,
                 result.grid_price,
                 result.position_size,
                 result.total_position,
                 result.average_price);
    }
    println!("{:-<60}", "");
}

fn run_calculation(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    config.validate()?;
    
    let grid_type = parse_grid_type(&config.base.grid_type)?;
    let position_mode = parse_position_mode(&config.position.mode)?;
    let multiplier = config.position.multiplier.unwrap_or(1.0);
    
    let mut calculator = GridCalculator::new(
        config.base.initial_price,
        grid_type,
        position_mode,
        config.position.base_size,
        multiplier,
    );
    
    print_calculation_header(
        config.base.initial_price,
        &config.base.grid_type,
        &config.position.mode,
        config.position.base_size,
        config.position.multiplier,
    );
    
    print_calculation_results(&mut calculator, &config.grid.levels);
    
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Calculate {
            price,
            grid_type,
            levels,
            mode,
            size,
            multiplier,
        } => {
            let grid_levels: Vec<f64> = levels
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            if grid_levels.is_empty() {
                eprintln!("Error: No valid grid levels provided");
                std::process::exit(1);
            }

            let mut calculator = GridCalculator::new(
                price,
                grid_type.into(),
                mode.into(),
                size,
                multiplier,
            );

            print_calculation_header(
                price,
                &format!("{:?}", grid_type),
                &format!("{:?}", mode),
                size,
                if matches!(mode, PositionModeArg::CurrentMultiple | PositionModeArg::IncrementMultiple) {
                    Some(multiplier)
                } else {
                    None
                },
            );
            
            print_calculation_results(&mut calculator, &grid_levels);
        }
        
        Commands::FromConfig { config, strategy } => {
            let cfg = match Config::from_file(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config file: {}", e);
                    std::process::exit(1);
                }
            };
            
            if let Some(strategy_name) = strategy {
                // Use specific strategy
                if let Some(strategies) = &cfg.strategies {
                    if let Some(strat) = strategies.iter().find(|s| s.name == strategy_name) {
                        if let Err(e) = strat.validate() {
                            eprintln!("Error validating strategy: {}", e);
                            std::process::exit(1);
                        }
                        let strategy_config = strat.to_config();
                        if let Err(e) = run_calculation(&strategy_config) {
                            eprintln!("Error running calculation: {}", e);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Strategy '{}' not found in config", strategy_name);
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("No strategies defined in config file");
                    std::process::exit(1);
                }
            } else {
                // Use main config
                if let Err(e) = run_calculation(&cfg) {
                    eprintln!("Error running calculation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::ListStrategies { config } => {
            let cfg = match Config::from_file(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading config file: {}", e);
                    std::process::exit(1);
                }
            };
            
            println!("\n=== Available Strategies ===\n");
            
            // Show main config
            println!("Main Configuration:");
            println!("  Grid Type: {}", cfg.base.grid_type);
            println!("  Position Mode: {}", cfg.position.mode);
            println!("  Levels: {:?}", cfg.grid.levels);
            
            // Show strategies
            if let Some(strategies) = &cfg.strategies {
                println!("\nNamed Strategies:");
                for strat in strategies {
                    println!("\n  Strategy: '{}'", strat.name);
                    println!("    Grid Type: {}", strat.grid_type);
                    println!("    Position Mode: {}", strat.position_mode);
                    println!("    Levels: {:?}", strat.levels);
                    if let Some(mult) = strat.multiplier {
                        println!("    Multiplier: {:.2}x", mult);
                    }
                }
            } else {
                println!("\nNo named strategies defined.");
            }
        }
    }
}