use crate::model::glob_side::*;
use crate::model::glob_side::Pair as OtherPair;
mod model;
use crate::model::explicit::value;
use crate::model::glob_side::nested::*;

mod exports {
    pub use crate::model::glob_side::*;
    pub use crate::model::glob_side::Pair as PublicPair;
}

struct Pair { value: u32 }

fn main() {
    let imported: crate::model::glob_side::Pair = Pair(2);
    let local = Pair { value: 4 };
    let OtherPair(other) = OtherPair(8);
    assert_eq!(model::glob_side::value(), 1);
    assert_eq!(exports::Pair(1).0, 1);
    assert_eq!(exports::PublicPair(8).0, other);
    assert_eq!(exports::value(), 1);
    let total = imported.0 + local.value + other + value() + 3_u32.score() + nested_value();
    println!("{total}");
}
