mod model;

pub mod extra;

use core::mem::size_of;
use std::num::NonZeroU32;

pub use crate::model::Public;
use crate::{model::root_value, outer::public_value};

mod outer {
    use std::num::NonZeroU16;

    use self::super::model::qualified_value;
    pub(self) use self::helper::self_value;
    pub(super) use super::model::parent_value;
    pub use crate::model::Public;
    pub(crate) use crate::model::crate_value;
    pub(in crate::outer) use crate::model::local_value;

    mod helper {
        use core::num::NonZeroU8;

        use super::super::model::child_value;

        pub fn self_value() -> u32 {
            child_value() + u32::from(NonZeroU8::new(1).unwrap().get())
        }
    }

    pub fn public_value() -> u32 {
        local_value() + self_value() + qualified_value()
            + u32::from(NonZeroU16::new(64).unwrap().get())
    }
}

fn main() {
    let Public(value) = outer::Public(128);
    let total = root_value() + outer::parent_value() + outer::crate_value()
        + public_value() + NonZeroU32::new(256).unwrap().get()
        + size_of::<u32>() as u32 + extra::value() + value;
    println!("{total}");
}
