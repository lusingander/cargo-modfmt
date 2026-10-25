use crate::Z;
mod z;
mod a;

fn boundary() {}

use crate::B;
use std::io;
mod b;

mod nested {
    use crate::Z;
    use std::fmt;
}
