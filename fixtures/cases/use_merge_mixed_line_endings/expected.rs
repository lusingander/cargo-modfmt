use foo::B;
use foo::A;

fn mixed_run() {}

use foo::{
    B,
    A,
};
use foo::C;

fn mixed_tree() {}

use foo::{a::A, z::Z};
