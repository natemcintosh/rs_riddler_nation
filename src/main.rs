use std::{collections::HashMap, time::Instant};

use clap::{App, Arg};
use itertools::Itertools;
use rand::Rng;

#[macro_use]
extern crate log;

const NUM_CASTLES: usize = 10;
const NUM_SPLITS: usize = NUM_CASTLES - 1;

// generate_uniform_random_distribution will create 10 numbers, between 0.0 and 100.0,
// which sum to 100.0.
fn generate_uniform_random_distribution() -> [i16; NUM_CASTLES] {
    split_points_to_array(&gen_uniform_random_split_points())
}

fn gen_uniform_random_split_points() -> [i16; NUM_SPLITS] {
    // To ensure they sum to 100.0, first generate 9 numbers between 0.0 and 100.0.
    // These will be the "splitting points", and the difference between all of them will
    // be the number of troops to send to that castle.
    let mut rng = rand::thread_rng();
    let mut split_points = [0_i16; NUM_SPLITS];

    // Fill the array with random numbers between 0.0 and 100.0.
    for i in 0..split_points.len() {
        split_points[i] = rng.gen_range(0..=100);
    }

    // Sort the split_points, so that the numbers are in ascending order.
    split_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Round all of the numbers to 1 decimal place
    // for i in 0..split_points.len() {
    //     split_points[i] = split_points[i].trunc() + (split_points[i].fract() * 10.0).trunc() / 10.0;
    // }

    split_points
}

// split_points_to_array takes a [i16; NUM_SPLITS] array of split points, and converts it to a
// [i16; NUM_CASTLES] array of the distances between the split points.
fn split_points_to_array(split_points: &[i16; NUM_SPLITS]) -> [i16; NUM_CASTLES] {
    // Calculate the difference between each number and the one before it. The first
    // number in this array is just the first split point, and the last number is
    // 100.0 - the last split point.
    let first_val = split_points[0];
    let last_val = 100 - split_points[split_points.len() - 1];
    let middle_vals = split_points.windows(2).map(|w| w[1] - w[0]);

    // Put all the values together into an array of length 10
    let mut result = [0_i16; NUM_CASTLES];
    result[0] = first_val;
    result[9] = last_val;
    for (i, val) in middle_vals.enumerate() {
        result[i + 1] = val;
    }

    result
}

// array_to_split_points will take a [i16; NUM_CASTLES] array of distances between split points,
// and convert it to a [i16; NUM_SPLITS] array of split points.
fn array_to_split_points(distribution: [i16; NUM_CASTLES]) -> [i16; NUM_SPLITS] {
    let mut split_points = [0_i16; NUM_SPLITS];
    for (idx, &item) in distribution.iter().enumerate() {
        if idx == 0 {
            split_points[idx] = item;
        } else if idx == 9 {
            split_points[idx - 1] = 100 - item;
        } else {
            split_points[idx] = split_points[idx - 1] + item;
        }
    }
    split_points
}

// generate_random_children will take in a [i16; NUM_CASTLES] array and create a set of children
// from it, with random mutations, about +-5 per castle
fn generate_random_children(
    arr: [i16; NUM_CASTLES],
    n_children: usize,
    variance_range: i16,
) -> Vec<[i16; NUM_CASTLES]> {
    let mut rng = rand::thread_rng();
    let mut children_splits = Vec::new();

    // Get the split points of the parent
    let split_points = array_to_split_points(arr);

    for _ in 0..n_children {
        let mut child_splits = split_points;
        for i in 0..child_splits.len() {
            let is_positive = rng.gen::<i32>().is_positive();
            let new_num = match is_positive {
                true => child_splits[i] + rng.gen_range(0..variance_range),
                false => child_splits[i] - rng.gen_range(0..variance_range),
            };

            // Make sure the new number is between 0.0 and 100.0
            if new_num < 0 {
                child_splits[i] = 0;
            } else if new_num > 100 {
                child_splits[i] = 100;
            } else {
                child_splits[i] = new_num;
            }
        }

        // Sort the split_points, so that the numbers are in ascending order.
        child_splits.sort_by(|a, b| a.partial_cmp(b).unwrap());

        children_splits.push(child_splits);
    }

    // Convert the children to a [i16; NUM_CASTLES] array
    children_splits
        .iter()
        .map(|child| split_points_to_array(child))
        .collect()
}

// sim will compare two length 10 arrays and see who wins
fn battle(p1: [i16; NUM_CASTLES], p2: [i16; NUM_CASTLES]) -> (f32, f32) {
    let mut p1_score = 0_f32;
    let mut p2_score = 0_f32;

    for (castle_num, (p1, p2)) in p1.iter().zip(p2.iter()).enumerate() {
        let score = (castle_num + 1) as f32;
        if p1 > p2 {
            p1_score += score;
        } else if p2 > p1 {
            p2_score += score;
        } else {
            p1_score += score / 2.0;
            p2_score += score / 2.0;
        }
    }

    (p1_score, p2_score)
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BattleScore {
    wins: u32,
    ties: u32,
    losses: u32,
}

impl BattleScore {
    fn new() -> Self {
        BattleScore {
            wins: 0,
            losses: 0,
            ties: 0,
        }
    }
}

// run_sims takes in a bunch of players, and returns some number of the best players
fn run_sims(
    players: &[[i16; NUM_CASTLES]],
    num_to_return: usize,
) -> Vec<([i16; NUM_CASTLES], BattleScore)> {
    // Create a HashMap to store the player's index and their score
    let mut result_map: HashMap<usize, BattleScore> = (0..players.len())
        .map(|idx| (idx, BattleScore::new()))
        .collect();

    // For each combination of two players, run a simulation, and store the result in the
    // result
    players
        .iter()
        .enumerate()
        .combinations(2)
        .for_each(|players| {
            let (idx1, p1) = players[0];
            let (idx2, p2) = players[1];
            let (p1_score, p2_score) = battle(*p1, *p2);
            if p1_score > p2_score {
                let mut winner_score = result_map[&idx1];
                winner_score.wins += 1;
                result_map.insert(idx1, winner_score);

                let mut loser_score = result_map[&idx2];
                loser_score.losses += 1;
                result_map.insert(idx2, loser_score);
            } else if p2_score > p1_score {
                let mut winner_score = result_map[&idx2];
                winner_score.wins += 1;
                result_map.insert(idx2, winner_score);

                let mut loser_score = result_map[&idx1];
                loser_score.losses += 1;
                result_map.insert(idx1, loser_score);
            } else {
                // It was a tie
                let mut p1score = result_map[&idx1];
                p1score.ties += 1;
                result_map.insert(idx1, p1score);

                let mut p2score = result_map[&idx2];
                p2score.ties += 1;
                result_map.insert(idx2, p2score);
            }
        });

    // Sort the results, and return the top num_to_return results
    result_map
        .iter()
        .sorted_by(|(_, p1), (_, p2)| p2.cmp(p1))
        .map(|(idx, &n_wins)| (players[*idx], n_wins))
        .take(num_to_return)
        .collect()
}

fn main() {
    // Start up a log
    env_logger::init();

    // Create the clap App
    let matches = App::new("rs_battle_for_nation")
        .version("0.1.0")
        .author("Nathan McIntosh")
        .about("A tool to do generational improvement for the Riddler Nation Game")
        .arg(
            Arg::with_name("num_generations")
                .short("g")
                .long("num_generations")
                .help("How many generations should be run")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num_children")
                .short("c")
                .long("num_children")
                .help("How many children should be spawned from each winner")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num_to_keep")
                .short("k")
                .long("num_to_keep")
                .help("How many winners should be kept from each round")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size_distribution")
                .short("s")
                .long("size_distribution")
                .help("How wide of a range, plus and minus, to go from each split point")
                .takes_value(true),
        )
        .get_matches();

    // Get the number of generations to run from matches
    let n: usize = matches
        .value_of("num_generations")
        .unwrap_or("100")
        .parse()
        .expect("Could not parse the number of generations to run");

    // Get the number of children to generate each time
    let n_children: usize = matches
        .value_of("num_children")
        .unwrap_or("10")
        .parse()
        .expect("Could not parse the number of children to generate");

    // Get the number how many to keep each time
    let n_to_keep: usize = matches
        .value_of("num_to_keep")
        .unwrap_or("10")
        .parse()
        .expect("Could not parse the number of children to generate");

    // Get the size_distribution with which to vary the split points
    let size_distribution: i16 = matches
        .value_of("size_distribution")
        .unwrap_or("5.0")
        .parse()
        .expect("Could not parse the size distribution");

    let pool_size: usize = n_children * n_to_keep;
    info!("Pool size: {}", pool_size);

    // Create the initial population
    let mut players = (0..pool_size)
        .map(|_| generate_uniform_random_distribution())
        .collect::<Vec<_>>();

    let mut best_players = run_sims(&players, n_to_keep);

    // time how long it takes to do n iterations
    let starttime = Instant::now();
    for gen_number in 0..n {
        // Print out the generation if it's a multiple of 500
        if gen_number % 500 == 0 {
            info!("Generation {}", gen_number);
        }

        best_players = run_sims(&players, n_to_keep);
        // Generate n_children random children from the best players
        players = best_players
            .iter()
            .flat_map(|(p, _)| generate_random_children(*p, n_children, size_distribution))
            .collect::<Vec<_>>();
    }
    // Print out how long each iteration took
    info!(
        "Each generation took: {} ms",
        starttime.elapsed().as_secs_f64() / 1e-3 / (n as f64)
    );

    // Print the best players
    info!("{:?}", best_players);
}

#[test]
fn test_split_to_array_round_trip() {
    // We'll test that converting a split point array to an array of distances and back
    // to a split point array is the same as the original array.
    // We do this 10_000 times, to make sure that the randomness is working.
    for _ in 0..10000 {
        let split_points = gen_uniform_random_split_points();
        let distances = split_points_to_array(&split_points);
        let split_points_back = array_to_split_points(distances);
        // Iterate over split_points and split_points_back, and make sure they are the same, to some level of precision.
        assert_eq!(split_points, split_points_back);
    }
}
