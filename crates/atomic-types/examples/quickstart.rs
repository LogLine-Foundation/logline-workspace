//! Quick-start example for `atomic-types` crypto primitives.

use atomic_types::{Cid32, Intent};

fn main() {
    // Cid32 serializes to/from lowercase hex
    let cid = Cid32([0x11; 32]);
    println!("cid hex = {cid}");

    // Intent normalizes whitespace deterministically
    let intent = Intent::from_raw(" allow  payment  up to  $5 ");
    println!(
        "canonical = {:?}",
        std::str::from_utf8(intent.as_bytes()).unwrap()
    );
}
