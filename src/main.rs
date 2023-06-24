#![feature(test)]

mod core;
mod final_battle;
mod seventh_battle;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// How many to run at once
    #[arg(short, long, default_value_t = 10_000)]
    pool_size: usize,
}

fn main() {
    let args = Args::parse();

    // Set up the player pool
    println!("Pool size: {}", args.pool_size);
    let pool: Vec<[i16; 10]> = (0..args.pool_size)
        .into_iter()
        .map(|_| core::generate_uniform_random_distribution())
        .collect();

    let res = final_battle::tournament(&pool);
    let winner = res.last().expect("The tournament produced an empty vector");
    println!("{:?}", winner);
}
