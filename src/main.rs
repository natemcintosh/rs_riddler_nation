#![feature(test)]

mod core;
mod final_battle;
mod seventh_battle;

use clap::Parser;
use rayon::prelude::*;

/// The idea for this program is to create a set of half-decent troop distributions by
/// running many tournaments on uniform randomly troop distributions. The hope is that
/// the winners of those tournaments will be similar to the troop distributions that
/// people submit for the contest. Then run a tournament for all of those troop
/// distributions.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// How many to compare at once
    #[arg(short, long, default_value_t = 500)]
    tournament_size: usize,

    /// How many tournaments to run
    #[arg(short, long, default_value_t = 10_000)]
    n_tournaments: usize,
}

fn create_pool(n_competitors: usize) -> Vec<[i16; 10]> {
    (0..n_competitors)
        .map(|_| core::generate_uniform_random_distribution())
        .collect()
}

fn main() {
    let start_time = std::time::Instant::now();
    let args = Args::parse();
    println!("Setting up tournaments");

    // Set up and run `n_tournaments`
    let winners: Vec<[i16; 10]> = (0..args.n_tournaments)
        // Start up tournaments in parallel
        .into_par_iter()
        // Create the uniform random pools
        .map(|_| create_pool(args.tournament_size))
        // Run all the tournaments
        .map(|players| final_battle::tournament(&players, false))
        // Get the best performer of each
        .map(|res| *res.last().expect("The tournament produced an empty vector"))
        .collect();

    println!("Running final tournament of winners of small tournaments");

    // Finally, run a tournament with all the winners
    let res = final_battle::tournament(&winners, true);
    let winner: &[i16; 10] = res.last().expect("The tournament produced an empty vector");

    println!("Final winner is {:?}", winner);
    println!("Run time was {}s", start_time.elapsed().as_secs());
}
