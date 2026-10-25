use foo::{z::Z, self as Zed, B, self, a::{Z, self as Nested, A}, self as Alpha, self as _};

fn nested_self() {}

use foo::a::self as Original;

fn trailing_self() {}

use foo::a::{self as Only};

fn singleton_self() {}

use foo::{b::self as Bee, a::self as Inner, Z};
