mod model;

use std::{
    mem::size_of,
    num::{NonZeroU16, NonZeroU32},
};

pub use crate::model::{Public, exported};
use crate::{
    model::{
        a::{first, second},
        z::z_value,
    },
    outer::value,
};

mod outer {
    pub(in crate::outer) use self::helper::{left, right};
    pub(super) use super::model::{crate_value, z::z_value};

    mod helper {
        use super::super::model::a::{first, second};

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
