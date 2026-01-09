use json_atomic::trajectory_confidence;

fn main() {
    let a = [0.1, 0.3, 0.6];
    let b = [0.09, 0.31, 0.6];
    let conf = trajectory_confidence(&a, &b);
    println!("confidence={:.4}", conf);
}
