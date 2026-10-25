mod model;

use crate::model::{
    Pair, Pair as First, Pair as Second, Score as _,
    branch::{self as Branch, child as renamed_child, value},
};

mod exports {
    pub(crate) use crate::model::{Pair as CratePair, branch::value as crate_value};
    pub use crate::model::{
        Pair as PublicPair,
        branch::{self as PublicBranch},
    };

    pub fn total() -> u32 {
        let CratePair(value) = CratePair(16);
        value + crate_value()
    }
}

fn main() {
    let Pair(a) = Pair(1);
    let First(b) = First(2);
    let Second(c) = Second(4);
    let exports::PublicPair(d) = exports::PublicPair(8);
    let total = a + b + c + d + Branch::value() + renamed_child() + value()
        + 3_u32.score() + exports::PublicBranch::child() + exports::total();
    println!("{total}");
}
