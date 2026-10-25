use foo::nested::*;
use foo::B;
use foo::{self as Root, *};
use foo::nested::{self as Nested, A};

fn nested_globs() {}

use foo::a::*;
use foo::a::A;
