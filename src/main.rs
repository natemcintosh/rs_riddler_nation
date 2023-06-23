#![feature(test)]

extern crate test;

use std::{cmp::Ordering, convert::TryInto, time::Instant, usize};

use clap::{App, Arg};
use itertools::Itertools;
use num::{
    bigint::{BigInt, ToBigInt},
    traits::One,
};
use rand::{seq::SliceRandom, Rng};

#[macro_use]
extern crate log;

const NUM_CASTLES: usize = 10;
const NUM_SPLITS: usize = NUM_CASTLES - 1;

/// generate_uniform_random_distribution will create 10 numbers, between 0.0 and 100.0,
/// which sum to 100.0.
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

/// split_points_to_array takes a [i16; NUM_SPLITS] array of split points, and converts it to a
/// [i16; NUM_CASTLES] array of the distances between the split points.
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

/// array_to_split_points will take a [i16; NUM_CASTLES] array of distances between split points,
/// and convert it to a [i16; NUM_SPLITS] array of split points.
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

/// generate_random_children will take in a [i16; NUM_CASTLES] array and create a set of children
/// from it, with random mutations, +-`variance_range` per castle
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
    children_splits.iter().map(split_points_to_array).collect()
}

/// sim will compare two length 10 arrays and see who wins
fn battle(p1: [i16; NUM_CASTLES], p2: [i16; NUM_CASTLES]) -> (f32, f32) {
    let mut p1_score = 0_f32;
    let mut p2_score = 0_f32;

    for (castle_num, (p1, p2)) in p1.iter().zip(p2.iter()).enumerate() {
        let score = (castle_num + 1) as f32;
        match p1.cmp(p2) {
            Ordering::Greater => p1_score += score,
            Ordering::Less => p2_score += score,
            Ordering::Equal => {
                p1_score += score / 2.0;
                p2_score += score / 2.0;
            }
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
    num_to_return: Option<usize>,
) -> Vec<([i16; NUM_CASTLES], BattleScore)> {
    // Create a HashMap to store the player's index and their score
    let mut results: Vec<BattleScore> = vec![BattleScore::new(); players.len()];

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
                results[idx1].wins += 1;
                results[idx2].losses += 1;
            } else if p2_score > p1_score {
                results[idx2].wins += 1;
                results[idx1].losses += 1;
            } else {
                // It was a tie
                results[idx1].ties += 1;
                results[idx2].ties += 1;
            }
        });

    // Sort the results, and return the top num_to_return results if asked for
    let n_take = match num_to_return {
        Some(n) => n,
        None => players.len(),
    };
    results
        .iter()
        .enumerate()
        .sorted_by(|p1, p2| p1.1.wins.cmp(&p2.1.wins))
        .map(|(idx, &battle_score)| (players[idx], battle_score))
        .take(n_take)
        .collect()
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

fn log_2(x: usize) -> u32 {
    num_bits::<usize>() as u32 - x.leading_zeros() - 1
}

/// Assumes that players are sorted. Use their indices as weights. The sum of two players'
/// weights should be equal for all battles at a given level
/// Once players are seeded in pairs, you should always be able to just take the next
/// two winners, and the weights should match automatically
fn seed_players(n_players: usize) -> Vec<usize> {
    // Assume that n_players is a power of 2
    // Start with 1 vs. 2, and work out from there. Each subsequent level should have
    // equal weights
    let n_rounds = log_2(n_players);

    // Allocate an array of length `n_players`
    let mut seeds: Vec<u32> = Vec::with_capacity(n_players);
    seeds.push(1);
    seeds.push(2);

    // Loop over each level of the tournament, start from the finals
    for r in 2..=n_rounds {
        let n_this_level = 2_u32.pow(r);
        let desired_sum = n_this_level + 1;
        seeds = seeds.iter().flat_map(|&x| [x, desired_sum - x]).collect();
    }
    seeds
        .iter()
        .map(|&x| x.try_into().expect("could not go from u32 to usize"))
        .map(|x: usize| x - 1)
        .collect()
}

fn next_power_of_2_after(x: usize) -> Option<usize> {
    for p in 1..50 {
        match 2_usize.pow(p).cmp(&x) {
            Ordering::Less => continue,
            Ordering::Equal | Ordering::Greater => return Some(2_usize.pow(p)),
        }
    }
    None
}

fn sort_according_to_inds<T: Copy>(items: &[T], inds: &[usize]) -> Vec<T> {
    let mut result = Vec::with_capacity(inds.len());
    for idx in inds {
        result.push(items[*idx]);
    }
    result
}

/// Runs a single elimination tournament, assuming the items in `pool` are sorted in
/// the correct order
fn run_tournament(pool: &[[i16; NUM_CASTLES]], n_top_keep: usize) -> Vec<[i16; NUM_CASTLES]> {
    let mut p: Vec<[i16; NUM_CASTLES]> = pool.to_owned();
    while p.len() > 1 {
        p = p
            .chunks(2)
            .map(|p| battle(p[0], p[1]))
            .map(|(p1_score, p2_score)| match p1_score.total_cmp(&p2_score) {
                Ordering::Less => p[1],
                Ordering::Equal => p[0],
                Ordering::Greater => p[0],
            })
            .collect();
    }
    return p;
}

/// For this simulation, each generation's pool is made up of
/// total = n_top_keep + (n_top_keep * n_children) + n_random + n_previous_tops
/// where n_top_keep is how many are carried over from the previous generation
/// n_children is how many children each of the top strategies from the previous generation
/// had.
/// n_random is how many random strategies to insert
/// n_previous_tops is how many previous top strategies to fetch from the database
///
/// The basic idea is to run many against eachother, use the results to seed a single
/// elimination tournament, and then report the top winners. Ideally also save the top
/// winners to a sqlite database for easy recall later. Then generate children of the
/// top performers, some random strategies, and pick out some previous winners (if any).
fn seventh_battle_for_riddler_nation(
    // n_generations: usize,
    // n_top_keep: usize,
    // n_children: usize,
    // n_previous_tops: usize,
    starting_size: usize,
) -> Vec<[i16; NUM_CASTLES]> {
    // Calculate how many random are needed to get to the next power of 2
    // let starting_size = n_top_keep + (n_top_keep * n_children) + n_previous_tops;
    let pool_size = match next_power_of_2_after(starting_size) {
        Some(n) => n,
        None => panic!("Could not figure out how large the pool size should be"),
    };
    println!("Pool size is {}", pool_size);
    let seeds = seed_players(pool_size);
    let pool: Vec<[i16; NUM_CASTLES]> = (0..pool_size)
        .into_iter()
        .map(|_| generate_uniform_random_distribution())
        .collect();
    let mut rng = rand::thread_rng();
    // Keep around 100 of the top performers
    let mut top_performers: Vec<_> = pool
        .choose_multiple(&mut rng, pool.len().min(100))
        .collect();

    let all_against_all_time = std::time::Instant::now();
    // Play them all against eachother, and get them back sorted worst to best
    let pool: Vec<_> = run_sims(&pool, None)
        .iter()
        .map(|(player, _)| *player)
        .collect();
    println!(
        "Running all strategies against eachother took {:0.6} s",
        all_against_all_time.elapsed().as_secs_f64()
    );

    // Now re-sort them according to the seed order
    let pool = sort_according_to_inds(&pool, &seeds);

    let tournament_time = std::time::Instant::now();
    // Run the tournament, which returns the top performers
    let new_top_performers: Vec<[i16; NUM_CASTLES]> = run_tournament(&pool, 1);
    println!(
        "Running tournament took {:0.6} us",
        tournament_time.elapsed().as_micros()
    );
    return new_top_performers;

    // Generate children, and random strategies
    // let new_kids: Vec<_> = new_top_performers
    //     .iter()
    //     .flat_map(|&tp| generate_random_children(tp, n_children, 1))
    //     .collect();

    // // Save the top performers
    // top_performers.shuffle(&mut rng);
    // let old_top_performers: Vec<_> = top_performers
    //     .drain(top_performers.len() - 1 - n_previous_tops..)
    //     .collect();
    // top_performers.extend(&new_top_performers);

    // Put them all into a new pool
    // let pool = new_top_performers.extend(&new_kids);
}

fn n_choose_k(n: u64, k: u64) -> BigInt {
    let mut res = BigInt::one();
    for i in 0..k {
        res = (res * (n - i).to_bigint().unwrap()) / (i + 1).to_bigint().unwrap();
    }
    res
}

fn main() {
    // Start up a log
    env_logger::init();

    // Create the clap App
    // let matches = App::new("rs_battle_for_nation")
    //     .version("0.1.0")
    //     .author("Nathan McIntosh")
    //     .about("A tool to do generational improvement for the Riddler Nation Game")
    //     .arg(
    //         Arg::with_name("num_generations")
    //             .short("g")
    //             .long("num_generations")
    //             .help("How many generations should be run")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::with_name("num_children")
    //             .short("c")
    //             .long("num_children")
    //             .help("How many children should be spawned from each winner")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::with_name("num_to_keep")
    //             .short("k")
    //             .long("num_to_keep")
    //             .help("How many winners should be kept from each round")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::with_name("size_distribution")
    //             .short("s")
    //             .long("size_distribution")
    //             .help("How wide of a range, plus and minus, to go from each split point")
    //             .takes_value(true),
    //     )
    //     .get_matches();

    // // Get the number of generations to run from matches
    // let n: usize = matches
    //     .value_of("num_generations")
    //     .unwrap_or("100")
    //     .parse()
    //     .expect("Could not parse the number of generations to run");

    // // Get the number of children to generate each time
    // let n_children: usize = matches
    //     .value_of("num_children")
    //     .unwrap_or("10")
    //     .parse()
    //     .expect("Could not parse the number of children to generate");

    // // Get the number how many to keep each time
    // let n_to_keep: usize = matches
    //     .value_of("num_to_keep")
    //     .unwrap_or("10")
    //     .parse()
    //     .expect("Could not parse the number of children to generate");

    // // Get the size_distribution with which to vary the split points
    // let size_distribution: i16 = matches
    //     .value_of("size_distribution")
    //     .unwrap_or("5.0")
    //     .parse()
    //     .expect("Could not parse the size distribution");

    let starting_size = 32769;
    let winner = seventh_battle_for_riddler_nation(starting_size);
    println!("{:?}", winner);
    // let pool_size: usize = n_children * n_to_keep;
    // info!("Pool size: {}", pool_size);
    // let report_number = 50000 / pool_size;

    // // Create the initial population
    // let mut players = (0..pool_size)
    //     .map(|_| generate_uniform_random_distribution())
    //     .collect::<Vec<_>>();

    // let mut best_players = run_sims(&players, Some(n_to_keep));

    // // time how long it takes to do n iterations
    // let starttime = Instant::now();
    // for gen_number in 0..n {
    //     // Print out the generation if it's a multiple of 500
    //     if gen_number % report_number == 0 {
    //         info!("Generation {}", gen_number);
    //     }

    //     best_players = run_sims(&players, Some(n_to_keep));
    //     // Generate n_children random children from the best players
    //     players = best_players
    //         .iter()
    //         .flat_map(|(p, _)| generate_random_children(*p, n_children, size_distribution))
    //         .collect::<Vec<_>>();
    // }
    // // Print out how long each iteration took
    // info!(
    //     "Each generation took: {} ms",
    //     starttime.elapsed().as_secs_f64() / 1e-3 / (n as f64)
    // );
    // info!(
    //     "Total time for {} battles was {} s",
    //     n * n_choose_k(pool_size as u64, 2),
    //     starttime.elapsed().as_secs_f64()
    // );

    // // Print the best players
    // info!("{:?}", best_players);
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::*;

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

    #[bench]
    fn bench_battle_close(b: &mut Bencher) {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [11, 9, 10, 10, 10, 10, 10, 10, 10, 10];

        assert_eq!(100, p1.iter().sum::<i16>(), "p1 sum did not equal 100");
        assert_eq!(100, p2.iter().sum::<i16>(), "p2 sum did not equal 100");

        b.iter(|| battle(p1, p2));
    }

    #[bench]
    fn bench_battle_even(b: &mut Bencher) {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];

        assert_eq!(100, p1.iter().sum::<i16>(), "p1 sum did not equal 100");
        assert_eq!(100, p2.iter().sum::<i16>(), "p2 sum did not equal 100");

        b.iter(|| battle(p1, p2));
    }

    #[bench]
    fn bench_battle_not_close(b: &mut Bencher) {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [0, 1, 2, 3, 4, 16, 17, 18, 19, 20];

        assert_eq!(100, p1.iter().sum::<i16>(), "p1 sum did not equal 100");
        assert_eq!(100, p2.iter().sum::<i16>(), "p2 sum did not equal 100");

        b.iter(|| battle(p1, p2));
    }

    #[test]
    fn test_log_2() {
        let x = 32;
        assert_eq!(5, log_2(x));
    }

    #[test]
    fn test_seed_players() {
        // For 4, expect to get:
        let want = vec![1, 4, 2, 3];
        let got = seed_players(4);
        assert_eq!(want, got);

        // For 8, expect:
        let want = vec![1, 8, 4, 5, 2, 7, 3, 6];
        let got = seed_players(8);
        assert_eq!(want, got);
    }

    #[bench]
    fn bench_seed_1024(b: &mut Bencher) {
        b.iter(|| seed_players(1024))
    }

    #[bench]
    fn bench_seed_16384(b: &mut Bencher) {
        b.iter(|| seed_players(16384))
    }

    #[test]
    fn test_next_power_of_2_after() {
        let inputs = vec![(1, 2), (2, 2), (3, 4), (9, 16), (240, 256), (900, 1024)];
        for (input, want) in inputs {
            assert_eq!(Some(want), next_power_of_2_after(input));
        }
    }

    #[bench]
    fn bench_next_power_of_2_after_240(b: &mut Bencher) {
        b.iter(|| next_power_of_2_after(240));
    }

    #[bench]
    fn bench_next_power_of_2_after_900(b: &mut Bencher) {
        b.iter(|| next_power_of_2_after(900));
    }

    #[test]
    fn test_sort_according_to_inds() {
        let v = vec![4, 3, 2, 1];
        let inds = vec![3, 2, 1, 0];
        assert_eq!(vec![1, 2, 3, 4], sort_according_to_inds(&v, &inds));
    }

    #[test]
    fn test_tournament_1() {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [0, 10, 10, 10, 10, 10, 10, 10, 10, 20];
        let players = vec![p1, p2];
        let got = run_tournament(&players, 1);
        let want = vec![p2];
        assert_eq!(want, got);
    }

    #[test]
    fn test_tournament_2() {
        let p1: [i16; 10] = [0, 10, 10, 10, 10, 10, 10, 10, 10, 20];
        let p2: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p3: [i16; 10] = [19, 18, 17, 16, 15, 5, 4, 3, 2, 1];
        let p4: [i16; 10] = [20, 19, 18, 17, 16, 4, 3, 2, 1, 0];
        let players = vec![p1, p4, p3, p2];
        let got = run_tournament(&players, 1);
        let want = vec![p1];
        assert_eq!(want, got);
    }
}
