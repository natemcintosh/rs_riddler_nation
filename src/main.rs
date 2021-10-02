use std::{collections::HashMap, env, time::Instant};

use itertools::Itertools;
use rand::Rng;

// generate_uniform_random_distribution will create 10 numbers, between 0.o and 100.0,
// which sum to 100.0.
fn generate_uniform_random_distribution() -> [f64; 10] {
    split_points_to_array(&gen_uniform_random_split_points())
}

fn gen_uniform_random_split_points() -> [f64; 9] {
    // To ensure they sum to 100.0, first generate 9 numbers between 0.0 and 100.0.
    // These will be the "splitting points", and the difference between all of them will
    // be the number of troops to send to that castle.
    let mut rng = rand::thread_rng();
    let mut split_points = [0f64; 9];

    // Fill the array with random numbers between 0.0 and 100.0.
    for i in 0..split_points.len() {
        split_points[i] = rng.gen_range(0.0..=100.0);
    }

    // Sort the split_points, so that the numbers are in ascending order.
    split_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Round all of the numbers to 1 decimal place
    for i in 0..split_points.len() {
        split_points[i] = split_points[i].trunc() + (split_points[i].fract() * 10.0).trunc() / 10.0;
    }

    split_points
}

// split_points_to_array takes a [f64; 9] array of split points, and converts it to a
// [f64; 10] array of the distances between the split points.
fn split_points_to_array(split_points: &[f64; 9]) -> [f64; 10] {
    // Calculate the difference between each number and the one before it. The first
    // number in this array is just the first split point, and the last number is
    // 100.0 - the last split point.
    let first_val = split_points[0];
    let last_val = 100.0 - split_points[split_points.len() - 1];
    let middle_vals = split_points.windows(2).map(|w| w[1] - w[0]);

    // Put all the values together into an array of length 10
    let mut result = [0f64; 10];
    result[0] = first_val;
    result[9] = last_val;
    for (i, val) in middle_vals.enumerate() {
        result[i + 1] = val;
    }

    result
}

// array_to_split_points will take a [f64; 10] array of distances between split points,
// and convert it to a [f64; 9] array of split points.
fn array_to_split_points(distribution: [f64; 10]) -> [f64; 9] {
    let mut split_points = [0f64; 9];
    for (idx, &item) in distribution.iter().enumerate() {
        if idx == 0 {
            split_points[idx] = item;
        } else if idx == 9 {
            split_points[idx - 1] = 100.0 - item;
        } else {
            split_points[idx] = split_points[idx - 1] + item;
        }
    }
    split_points
}

// generate_random_children will take in a [f64; 10] array and create a set of children
// from it, with random mutations, about +-5 per castle
fn generate_random_children(arr: [f64; 10], n_children: usize) -> Vec<[f64; 10]> {
    let mut rng = rand::thread_rng();
    let mut children_splits = Vec::new();

    // Get the split points of the parent
    let split_points = array_to_split_points(arr);

    for _ in 0..n_children {
        let mut child_splits = split_points.clone();
        for i in 0..child_splits.len() {
            let new_num = child_splits[i] + rng.gen_range(-5.0..5.0);
            // Make sure the new number is between 0.0 and 100.0
            if new_num < 0.0 {
                child_splits[i] = 0.0;
            } else if new_num > 100.0 {
                child_splits[i] = 100.0
            } else {
                child_splits[i] = new_num;
            }
        }

        // Sort the split_points, so that the numbers are in ascending order.
        child_splits.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Round all of the numbers to 1 decimal place
        for i in 0..child_splits.len() {
            child_splits[i] =
                child_splits[i].trunc() + (child_splits[i].fract() * 10.0).trunc() / 10.0;
        }
        children_splits.push(child_splits);
    }

    // Convert the children to a [f64; 10] array
    children_splits
        .iter()
        .map(|child| split_points_to_array(child))
        .collect()
}

// sim will compare two length 10 arrays and see who wins
fn p1_wins(p1: [f64; 10], p2: [f64; 10]) -> bool {
    let mut p1_score = 0f64;
    let mut p2_score = 0f64;

    for (castle_num, (p1, p2)) in p1.iter().zip(p2.iter()).enumerate() {
        if p1 > p2 {
            p1_score += castle_num as f64;
        } else if p2 > p1 {
            p2_score += castle_num as f64
        } else {
            p1_score += castle_num as f64 / 2.0;
            p2_score += castle_num as f64 / 2.0;
        }
    }

    p1_score > p2_score
}

// run_sims takes in a bunch of players, and returns some number of the best players
fn run_sims(players: &[[f64; 10]], num_to_return: usize) -> Vec<([f64; 10], usize)> {
    // Create a HashMap to store the player's index and their score
    let mut result_map: HashMap<usize, usize> = (0..players.len()).map(|idx| (idx, 0)).collect();

    // For each combination of two players, run a simulation, and store the result in the
    // result
    players
        .iter()
        .enumerate()
        .combinations(2)
        .for_each(|players| {
            let (idx1, p1) = players[0];
            let (idx2, p2) = players[1];
            if p1_wins(*p1, *p2) {
                result_map.insert(idx1, result_map[&idx1] + 1);
            } else {
                result_map.insert(idx2, result_map[&idx2] + 1);
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
    // Collect the command line arguments
    let args: Vec<String> = env::args().collect();
    // The first argument is the number of generations to run
    let n: usize = args[1].parse().unwrap_or(1000);
    // The second argument is the size of the pool to test each time
    let pool_size: usize = args[2].parse().unwrap_or(100);
    // The third argument is the number of children to generate each time
    let n_children: usize = args[3].parse().unwrap_or(10);

    // Generate 100 random players
    let mut players = (0..pool_size)
        .map(|_| generate_uniform_random_distribution())
        .collect::<Vec<_>>();

    let mut best_players = run_sims(&players, 10);

    // time how long it takes to do n iterations
    let starttime = Instant::now();
    for gen_number in 0..n {
        // Print out the generation if it's a multiple of 500
        if gen_number % 500 == 0 {
            println!("Generation {}", gen_number);
        }

        best_players = run_sims(&players, 10);
        // Generate 10 random children from the best players, and repeat 10 ten times
        players = best_players
            .iter()
            .map(|(p, _)| generate_random_children(*p, n_children))
            .flatten()
            .collect::<Vec<_>>();
    }
    // Print out how long each iteration took
    println!(
        "Each iteration took: {} ms",
        starttime.elapsed().as_millis() as f64 / n as f64
    );

    // Print the best players
    println!("{:?}", best_players);
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
        for (p1, p2) in split_points.iter().zip(split_points_back.iter()) {
            assert!((p1 - p2).abs() < 0.00000001);
        }
    }
}
