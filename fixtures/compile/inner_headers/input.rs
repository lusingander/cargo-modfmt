#![allow(dead_code)]
//! Crate documentation.
#![doc = "Additional documentation."]
#![cfg(all())]

use std::path::Path;
use crate::data::second as Second;
mod data;
use std::collections::BTreeMap;
use crate::data::first as First;

#[cfg(all())]
mod worker {
    /*! Module documentation. */
    #![allow(dead_code)]
    #![doc(hidden)]
    #![cfg(all())]

    use crate::data::second;
    use crate::data::first;

    pub fn value() -> usize { first() + second() }
}

mod conditional {
    #![cfg(test)]
    //! Conditional module documentation.
    use crate::data::second;
    use crate::data::first;

    pub fn value() -> usize { second() - first() + 3 }
}

fn main() {
    let total = First() + Second() + worker::value()
        + Path::new("abc").as_os_str().len() + BTreeMap::<u8, u8>::new().len();
    #[cfg(test)]
    let total = total + conditional::value();
    println!("{total}");
}
