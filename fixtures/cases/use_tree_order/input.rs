use foo::{z::Z, a::Z, a::{Z, A}, a, a::{D, B}, alpha2, alpha10, Alpha, a, Å, a::A};

fn boundary() {}

use foo::z;
pub use foo::{Z};
use foo::{B, A};
use foo;
pub(crate) use foo::{D, C};
use foo::a;
use foo0::{B, A};
