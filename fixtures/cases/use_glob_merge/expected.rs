use foo::{
    self as Root, B,
    nested::{self as Nested, A, *},
    *,
};

fn nested_globs() {}

use foo::a::{A, *};
