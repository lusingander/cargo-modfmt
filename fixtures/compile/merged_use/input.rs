use std::num::NonZeroU8;
pub use crate::model::Public;
use crate::model::a::second;
mod model;
use std::mem::size_of;
pub use crate::model::exported;
use crate::model::{z::z_value, a::first};
use std::num::NonZeroU16;
use crate::outer::value;

mod outer {
    pub ( super ) use super::model::z::z_value;
    pub(super) use super::model::crate_value;
    pub(in crate::outer) use self::helper::right;
    pub(in crate::outer) use self::helper::left;
    use self::super::model::a::second;
    use self::super::model::a::first;

    mod helper {
        use super::super::model::a::{second};
        use super::super::model::a::first;

        pub fn left() -> u32 {
            first()
        }

        pub fn right() -> u32 {
            second()
        }
    }

    pub fn value() -> u32 {
        left() + right() + first() + second()
    }
}

fn main() {
    let Public(public) = Public(32);
    let total = exported() + first() + second() + z_value()
        + outer::z_value() + outer::crate_value() + value()
        + u32::from(NonZeroU8::new(64).unwrap().get())
        + u32::from(NonZeroU16::new(128).unwrap().get()) + size_of::<u8>() as u32 + public;
    println!("{total}");
}
