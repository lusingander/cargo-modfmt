use foo::{
    B,
    A,
};

fn inner_mixed() {}

mod z;
use foo::{B, A};
mod a;

fn run_mixed() {}

use foo::{
    a::{A, B},
    z::Z,
};
