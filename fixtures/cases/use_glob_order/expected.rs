use foo::{
    self as Root, B,
    a::*,
    a::{self as Module, A, C, *},
    z::{Z, *},
    *,
};

fn nested_order() {}

use foo::a::*;
