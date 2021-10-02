use rand::{self, Rng};

// generate_uniform_random_distribution will create 10 numbers, between 0.o and 100.0,
// which sum to 100.0.
fn generate_uniform_random_distribution() -> [f64; 10] {
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
        let trunced = split_points[i].trunc() + (split_points[i].fract() * 10.0).trunc() / 10.0;
        split_points[i] = trunced;
    }

    // Calculate the difference between each number and the one before it. The first
    // number in this vector is just the first split point, and the last number is
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

fn main() {
    // Generate three distributions
    let g1 = generate_uniform_random_distribution();
    let g2 = generate_uniform_random_distribution();
    let g3 = generate_uniform_random_distribution();

    println!("{:?}", g1);
    println!("{:?}", g2);
    println!("{:?}", g3);

    // Verify that the sum of each distribution is 100.0
    let sum1 = g1.iter().sum::<f64>();
    let sum2 = g2.iter().sum::<f64>();
    let sum3 = g3.iter().sum::<f64>();
    assert_eq!(sum1, 100.0);
    assert_eq!(sum2, 100.0);
    assert_eq!(sum3, 100.0);
}
