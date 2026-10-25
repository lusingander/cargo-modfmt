#![allow(dead_code)]
//! Crate documentation.
#![doc = "Additional documentation."]
#![cfg(all())]

mod data;

use std::{collections::BTreeMap, path::Path};

use crate::data::{first as First, second as Second};

#[cfg(all())]
mod worker {
    /*! Module documentation. */
    #![allow(dead_code)]
    #![doc(hidden)]
    #![cfg(all())]

    use crate::data::{first, second};

    pub fn value() -> usize { first() + second() }
}

mod conditional {
    #![cfg(test)]
    //! Conditional module documentation.
    use crate::data::{first, second};

    pub fn value() -> usize { second() - first() + 3 }
}

fn main() {
    let total = First() + Second() + worker::value()
        + Path::new("abc").as_os_str().len() + BTreeMap::<u8, u8>::new().len();
    #[cfg(test)]
    let total = total + conditional::value();
    println!("{total}");
}
