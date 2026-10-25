use foo::{
    self, self as Alpha, self as Zed, self as _, B,
    a::{self as Nested, A, Z},
    z::Z,
};

fn nested_self() {}

use foo::a::{self as Original};

fn trailing_self() {}

use foo::a::{self as Only};

fn singleton_self() {}

use foo::{
    Z,
    a::{self as Inner},
    b::{self as Bee},
};
