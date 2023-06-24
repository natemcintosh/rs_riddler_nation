use crate::core;

/// Run all possible one-on-one matches. A victory is worth 1 battle point, while a tie
/// is worth 0.5 points. After all the one-on-one matchups are complete, whoever has
/// accumulated the fewest victory points will be eliminated from the tournament, after
/// which the battle will recommence with one fewer competitor.
///
/// Returns a vector of the players, from last place to first place (ascending order)
pub fn tournament(players: &[[i16; 10]]) -> Vec<[i16; 10]> {
    let mut pl: Vec<[i16; 10]> = players.to_owned();
    let mut res: Vec<[i16; 10]> = vec![];

    // Run the battles, remove the worst, run again
    for _ in 0..pl.len() {
        pl = core::run_battles(&pl, None)
            .iter()
            .map(|(troops, _)| *troops)
            .collect();
    }

    res
}
