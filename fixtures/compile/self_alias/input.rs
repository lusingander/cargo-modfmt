use crate::model::Pair as Second;
mod model;
use crate::model::branch::{value, self as Branch};
use crate::model::Pair;
use crate::model::branch::child as renamed_child;
use crate::model::Pair as First;
use crate::model::Score as _;

mod exports {
    pub use crate::model::branch::{self as PublicBranch};
    pub use crate::model::Pair as PublicPair;
    pub(crate) use crate::model::Pair as CratePair;
    pub ( crate ) use crate::model::branch::value as crate_value;

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
