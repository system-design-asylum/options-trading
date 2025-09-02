use options_trading::simulation::run_simulation;
use std::env;

fn main() {
    println!("🚀 Options Trading System - Market Simulation");

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "fast" {
        println!("Running demo mode (25 rounds, verbose)...\n");
        run_simulation(25, true);
    } else if args.len() > 1 && args[1] == "medium" {
        println!("Running ADVANCED simulation (50 rounds, 10 sophisticated traders, verbose)...\n");
        run_simulation(50, true);
    } else if args.len() > 1 && args[1] == "long" {
        println!("Running LONG simulation (100 rounds, marathon trading)...\n");
        run_simulation(100, true);
    } else if args.len() > 1 && args[1] == "insane" {
        println!("Running insane simulation (1000 rounds, marathon trading)...\n");
        run_simulation(1000, true);
    } else {
        println!("Usage: cargo run [demo|advanced|full|epic|fast]");
        println!("  demo: 25 rounds with 10 sophisticated traders");
        println!("  advanced: 50 rounds with full feature simulation");
        println!("  full: 50 rounds with verbose output");
        println!("  epic: 100 rounds marathon with all features");
        println!("  fast: 100 rounds with minimal output");
        println!("\nRunning default demo...\n");
        run_simulation(25, true);
    }
}
