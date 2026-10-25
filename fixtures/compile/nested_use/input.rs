use std::{num::{NonZeroU32, NonZeroU16}, mem::size_of};
pub use crate::model::{Public, exported};
mod model;
use crate::model::{z::z_value, a::{second, first}};
use crate::outer::value;

mod outer {
    pub(super) use super::model::{z::z_value, crate_value};
    pub(in crate::outer) use self::helper::{right, left};

    mod helper {
        use super::super::model::a::{second, first};

        pub fn left() -> u32 {
            first()
        }

        pub fn right() -> u32 {
            second()
        }
    }

    pub fn value() -> u32 {
        left() + right()
    }
}

fn main() {
    let Public(public) = Public(32);
    let total = exported() + first() + second() + z_value()
        + outer::z_value() + outer::crate_value() + value()
        + u32::from(NonZeroU16::new(64).unwrap().get())
        + NonZeroU32::new(128).unwrap().get() + size_of::<u8>() as u32 + public;
    println!("{total}");
}
