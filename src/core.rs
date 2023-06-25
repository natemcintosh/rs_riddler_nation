extern crate test;

use itertools::Itertools;
use rand::Rng;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

/// battle will compare two length 10 arrays and see who wins
pub fn battle(p1: [i16; 10], p2: [i16; 10]) -> (f32, f32) {
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

/// generate_uniform_random_distribution will create 10 numbers, between 0.0 and 100.0,
/// which sum to 100.0.
pub fn generate_uniform_random_distribution() -> [i16; 10] {
    split_points_to_array(&gen_uniform_random_split_points())
}

pub fn gen_uniform_random_split_points() -> [i16; 9] {
    // To ensure they sum to 100.0, first generate 9 numbers between 0.0 and 100.0.
    // These will be the "splitting points", and the difference between all of them will
    // be the number of troops to send to that castle.
    let mut rng = rand::thread_rng();
    let mut split_points = [0_i16; 9];

    // Fill the array with random numbers between 0.0 and 100.0.
    for sp in &mut split_points {
        *sp = rng.gen_range(0..=100);
    }

    // Sort the split_points, so that the numbers are in ascending order.
    split_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Round all of the numbers to 1 decimal place
    // for i in 0..split_points.len() {
    //     split_points[i] = split_points[i].trunc() + (split_points[i].fract() * 10.0).trunc() / 10.0;
    // }

    split_points
}

/// split_points_to_array takes a [i16; 9] array of split points, and converts it to a
/// [i16; 10] array of the distances between the split points.
pub fn split_points_to_array(split_points: &[i16; 9]) -> [i16; 10] {
    // Calculate the difference between each number and the one before it. The first
    // number in this array is just the first split point, and the last number is
    // 100.0 - the last split point.
    let first_val = split_points[0];
    let last_val = 100 - split_points[split_points.len() - 1];
    let middle_vals = split_points.windows(2).map(|w| w[1] - w[0]);

    // Put all the values together into an array of length 10
    let mut result = [0_i16; 10];
    result[0] = first_val;
    result[9] = last_val;
    for (i, val) in middle_vals.enumerate() {
        result[i + 1] = val;
    }

    result
}

/// array_to_split_points will take a [i16; 10] array of distances between split points,
/// and convert it to a [i16; 9] array of split points.
pub fn _array_to_split_points(distribution: [i16; 10]) -> [i16; 9] {
    let mut split_points = [0_i16; 9];
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

/// generate_random_children will take in a [i16; 10] array and create a set of children
/// from it, with random mutations, +-`variance_range` per castle
pub fn _generate_random_children(
    arr: [i16; 10],
    n_children: usize,
    variance_range: i16,
) -> Vec<[i16; 10]> {
    let mut rng = rand::thread_rng();
    let mut children_splits = Vec::new();

    // Get the split points of the parent
    let split_points = _array_to_split_points(arr);

    for _ in 0..n_children {
        let mut child_splits = split_points;
        for split_pt in &mut child_splits {
            let is_positive = rng.gen::<i32>().is_positive();
            let new_num = match is_positive {
                true => *split_pt + rng.gen_range(0..variance_range),
                false => *split_pt - rng.gen_range(0..variance_range),
            };

            // Make sure the new number is between 0.0 and 100.0
            if new_num < 0 {
                *split_pt = 0;
            } else if new_num > 100 {
                *split_pt = 100;
            } else {
                *split_pt = new_num;
            }
        }

        // Sort the split_points, so that the numbers are in ascending order.
        child_splits.sort_by(|a, b| a.partial_cmp(b).unwrap());

        children_splits.push(child_splits);
    }

    // Convert the children to a [i16; 10] array
    children_splits.iter().map(split_points_to_array).collect()
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BattleScore {
    pub wins: u32,
    pub ties: u32,
    pub losses: u32,
}

impl BattleScore {
    pub fn new() -> Self {
        BattleScore {
            wins: 0,
            losses: 0,
            ties: 0,
        }
    }
}

/// run_battles_slice takes in a bunch of players, and returns some number of the best
/// players in ascending order (last place first, winner has highest index)
pub fn _run_battles_slice(
    players: &[[i16; 10]],
    num_to_return: Option<usize>,
) -> Vec<([i16; 10], BattleScore)> {
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
        // Sort by most wins
        .sorted_by(|p1, p2| p1.1.wins.cmp(&p2.1.wins))
        // Get the troops that match this score
        .map(|(idx, &battle_score)| (players[idx], battle_score))
        .take(n_take)
        .collect()
}

fn bs_with_1_win() -> BattleScore {
    let mut bs = BattleScore::new();
    bs.wins += 1;
    bs
}

fn bs_with_1_loss() -> BattleScore {
    let mut bs = BattleScore::new();
    bs.losses += 1;
    bs
}

fn bs_with_1_tie() -> BattleScore {
    let mut bs = BattleScore::new();
    bs.ties += 1;
    bs
}

/// run_battles_set takes in a bunch of players, and returns some number of the best
/// players in ascending order (last place first, winner has highest index)
pub fn run_battles_set(players: &HashSet<[i16; 10]>) -> HashMap<[i16; 10], BattleScore> {
    // Create a HashMap to store the player's index and their score
    let mut results: HashMap<[i16; 10], BattleScore> = HashMap::with_capacity(players.len());

    // For each combination of two players, run a simulation, and store the result in the
    // result
    players.iter().combinations(2).for_each(|players| {
        let p1 = players[0];
        let p2 = players[1];
        let (p1_score, p2_score) = battle(*p1, *p2);
        if p1_score > p2_score {
            // Add a win, or create new entry if needed
            results
                .entry(*p1)
                .and_modify(|bs| bs.wins += 1)
                .or_insert_with(bs_with_1_win);
            // Add a loss, or create new entry if needed
            results
                .entry(*p2)
                .and_modify(|bs| bs.losses += 1)
                .or_insert_with(bs_with_1_loss);
        } else if p2_score > p1_score {
            results
                .entry(*p2)
                .and_modify(|bs| bs.wins += 1)
                .or_insert_with(bs_with_1_win);
            results
                .entry(*p1)
                .and_modify(|bs| bs.losses += 1)
                .or_insert_with(bs_with_1_loss);
        } else {
            // It was a tie
            results
                .entry(*p1)
                .and_modify(|bs| bs.ties += 1)
                .or_insert_with(bs_with_1_tie);
            results
                .entry(*p2)
                .and_modify(|bs| bs.ties += 1)
                .or_insert_with(bs_with_1_tie);
        }
    });
    results
}

#[cfg(test)]
mod tests {
    use std::vec;

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
            let split_points_back = _array_to_split_points(distances);
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
    fn test_run_battles() {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [100, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let p3: [i16; 10] = [90, 0, 0, 0, 0, 0, 0, 0, 0, 10];

        let players = vec![p1, p2, p3];
        let got: Vec<[i16; 10]> = _run_battles_slice(&players, None)
            .iter()
            .map(|(troops, _)| *troops)
            .collect();
        let want = vec![p2, p3, p1];
        assert_eq!(want, got);
    }

    #[bench]
    fn bench_run_battles_slice(b: &mut Bencher) {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [100, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let p3: [i16; 10] = [90, 0, 0, 0, 0, 0, 0, 0, 0, 10];

        let players = vec![p1, p2, p3];
        b.iter(|| _run_battles_slice(&players, None));
    }

    #[bench]
    fn bench_run_battles_set(b: &mut Bencher) {
        let p1: [i16; 10] = [10, 10, 10, 10, 10, 10, 10, 10, 10, 10];
        let p2: [i16; 10] = [100, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let p3: [i16; 10] = [90, 0, 0, 0, 0, 0, 0, 0, 0, 10];

        let players = HashSet::from([p1, p2, p3]);
        b.iter(|| run_battles_set(&players));
    }
}
