use std::cmp::Ordering;

use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::core::{self};

/// Run all possible one-on-one matches. A victory is worth 1 battle point, while a tie
/// is worth 0.5 points. After all the one-on-one matchups are complete, whoever has
/// accumulated the fewest victory points will be eliminated from the tournament, after
/// which the battle will recommence with one fewer competitor.
///
/// If two warlords are tied with the fewest number of victory points, the first
/// tiebreaker will be whoever has more wins (and fewer ties) and the second tiebreaker
/// will be performance in the preceding round (and then the round before that, etc.).
/// If two or more strategies on the chopping block are precisely the same, I will
/// randomly pick which one to eliminate.
///
/// Returns a vector of the players, from last place to first place (ascending order)
pub fn tournament(players: &[[i16; 10]], verbose: bool) -> Vec<[i16; 10]> {
    // If we make this a set, then if there are ever any players with the same troop
    // distribution, they will be combined into one player. Maybe that's fine for our
    // simulation, since they would get the same score in the end.
    // Maybe I want to use a BTreeSet bc they are ordered (would have to impl Ord). Will
    // investigate later.
    let mut pl: FxHashSet<[i16; 10]> = players.iter().copied().collect();
    let mut res: Vec<[i16; 10]> = Vec::with_capacity(players.len());

    // let mut n_ties: u32 = 0;
    let mut round: usize = 0;

    // Run the battles, remove the worst, run again
    // The below might iterate one more time than it should. Will probably error out if
    // it does
    while pl.len() > 2 {
        round += 1;
        // Run all one on one matches
        let scores = core::run_battles_set(&pl);

        let sorted: Vec<(&[i16; 10], u32)> = scores
            .iter()
            // Calc victory points: 1 for win, 0.5 for tie. (to avoid changing types,
            // double wins instead of halving ties)
            .map(|(troops, bs)| (troops, (2 * bs.wins) + bs.ties))
            // Sort by victory points
            .sorted_by(|a, b| a.1.cmp(&b.1))
            .collect();

        // If two warlords are tied with the fewest number of victory points, the first
        // tiebreaker will be whoever has more wins (and fewer ties) and the second
        // tiebreaker will be performance in the preceding round (and then the round
        // before that, etc.). If two or more strategies on the chopping block are
        // precisely the same, I will randomly pick which one to eliminate.
        // If the players at indices 0 and 1 have the same victory points
        if sorted[0].1 == sorted[1].1 {
            let p1_wins = scores.get(sorted[0].0).unwrap().wins;
            let p2_wins = scores.get(sorted[1].0).unwrap().wins;
            match p1_wins.cmp(&p2_wins) {
                Ordering::Equal => {
                    // If they have the same number of wins, warn, and pick the first
                    // n_ties += 1;
                    // Loser gets put in result vector
                    res.push(*sorted[0].0);
                    // And removed from `pl`
                    pl.remove(sorted[0].0);
                }
                Ordering::Less => {
                    // Drop p1
                    res.push(*sorted[0].0);
                    // And removed from `pl`
                    pl.remove(sorted[0].0);
                }
                Ordering::Greater => {
                    // Drop p2
                    res.push(*sorted[1].0);
                    // And removed from `pl`
                    pl.remove(sorted[1].0);
                }
            }
        } else {
            // Loser gets put in result vector
            res.push(*sorted[0].0);
            // And removed from `pl`
            pl.remove(sorted[0].0);
        }

        if verbose {
            println!("Finished round {round}");
        }
    }

    // Determine which of the two wins the one-on-oen battle
    let top_two: Vec<[i16; 10]> = pl.iter().copied().collect();
    let (p1_score, p2_score) = core::battle(top_two[0], top_two[1]);
    match p1_score.partial_cmp(&p2_score) {
        Some(o) => match o {
            Ordering::Less => {
                res.push(top_two[0]);
                res.push(top_two[1]);
            }
            Ordering::Equal => {
                res.push(top_two[0]);
                res.push(top_two[1]);
            }
            Ordering::Greater => {
                res.push(top_two[1]);
                res.push(top_two[0]);
            }
        },
        None => panic!("Battle score was NaN"),
    }

    // if n_ties > 0 {
    //     println!(
    //         "There were {} cases where the two losers had the same number of wins",
    //         n_ties
    //     );
    // }

    res
}
