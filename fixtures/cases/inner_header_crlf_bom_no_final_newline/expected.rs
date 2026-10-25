#![allow(/* Keep this. */ unused_imports)]
//! Crate documentation.

mod a;
mod z;

use std::{collections::BTreeMap, path::Path};

mod child {
    /*! Module documentation. */
    #![doc(hidden)]
    use std::io::{Read, Write};
}