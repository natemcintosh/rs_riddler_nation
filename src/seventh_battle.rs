use std::cmp::Ordering;

use crate::core;

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
fn run_tournament(pool: &[[i16; 10]]) -> Vec<[i16; 10]> {
    let mut p: Vec<[i16; 10]> = pool.to_owned();
    while p.len() > 1 {
        p = p
            .chunks(2)
            .map(|p| core::battle(p[0], p[1]))
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
pub fn seventh_battle_for_riddler_nation(
    // n_generations: usize,
    // n_top_keep: usize,
    // n_children: usize,
    // n_previous_tops: usize,
    starting_size: usize,
) -> Vec<[i16; 10]> {
    // Calculate how many random are needed to get to the next power of 2
    // let starting_size = n_top_keep + (n_top_keep * n_children) + n_previous_tops;
    let pool_size = match next_power_of_2_after(starting_size) {
        Some(n) => n,
        None => panic!("Could not figure out how large the pool size should be"),
    };
    println!("Pool size is {}", pool_size);
    let seeds = seed_players(pool_size);
    let pool: Vec<[i16; 10]> = (0..pool_size)
        .into_iter()
        .map(|_| core::generate_uniform_random_distribution())
        .collect();
    // let mut rng = rand::thread_rng();
    // Keep around 100 of the top performers
    // let top_performers: Vec<_> = pool
    //     .choose_multiple(&mut rng, pool.len().min(100))
    //     .collect();

    let all_against_all_time = std::time::Instant::now();
    // Play them all against eachother, and get them back sorted worst to best
    let pool: Vec<_> = core::run_battles(&pool, None)
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
    let new_top_performers: Vec<[i16; 10]> = run_tournament(&pool);
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

// fn n_choose_k(n: u64, k: u64) -> BigInt {
//     let mut res = BigInt::one();
//     for i in 0..k {
//         res = (res * (n - i).to_bigint().unwrap()) / (i + 1).to_bigint().unwrap();
//     }
//     res
// }

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    #[test]
    fn test_log_2() {
        let x = 32;
        assert_eq!(5, log_2(x));
    }

    #[test]
    fn test_seed_players() {
        // For 4, expect to get:
        let want = vec![0, 3, 1, 2];
        let got = seed_players(4);
        assert_eq!(want, got);

        // For 8, expect:
        let want = vec![0, 7, 3, 4, 1, 6, 2, 5];
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
        let got = run_tournament(&players);
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
        let got = run_tournament(&players);
        let want = vec![p1];
        assert_eq!(want, got);
    }
}
