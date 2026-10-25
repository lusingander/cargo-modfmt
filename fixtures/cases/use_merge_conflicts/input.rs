use foo::z::Z;
use bar::B;
use foo;
use foo::a::A;
use bar::A;

fn root_and_child() {}

use foo::z::Z;
use foo::a::B;
use foo::a;

fn leaf_and_child() {}

use foo::B;
use foo :: A;
use foo::A;
use foo::C;

fn repeated_path() {}

use foo::{B, a::A, a::{A}};
use foo::C;

fn repeated_element() {}

use foo::{z::Z, a::B, a};
use foo::c::C;

fn nested_leaf_and_child() {}

use foo;
pub use foo::z::Z;
pub use foo::a::A;

fn separate_visibility() {}

use foo::alpha::A;
use foo::a;

fn segment_prefix() {}

use foo::{a::B, a::A, b::{Only}};
