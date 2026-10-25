#![allow(/* Keep this. */ unused_imports)]
//! Crate documentation.

use std::path::Path;
mod z; mod a;
use std::collections::BTreeMap;

mod child {
    /*! Module documentation. */
    #![doc(hidden)]
    use std::io::Write;
    use std::io::Read;
}