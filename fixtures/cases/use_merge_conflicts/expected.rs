use bar::{A, B};
use foo;
use foo::a::A;
use foo::z::Z;

fn root_and_child() {}

use foo::a;
use foo::a::B;
use foo::z::Z;

fn leaf_and_child() {}

use foo::A;
use foo::A;
use foo::B;
use foo::C;

fn repeated_path() {}

use foo::C;
use foo::{B, a::A, a::A};

fn repeated_element() {}

use foo::c::C;
use foo::{a, a::B, z::Z};

fn nested_leaf_and_child() {}

use foo;
pub use foo::{a::A, z::Z};

fn separate_visibility() {}

use foo::{a, alpha::A};

fn segment_prefix() {}

use foo::{a::A, a::B, b::Only};
