use foo::{
    Alpha, a, a,
    a::A,
    a::Z,
    a::{A, Z},
    a::{B, D},
    alpha2, alpha10,
    z::Z,
    Å,
};

fn boundary() {}

use foo;
pub use foo::Z;
use foo::a;
use foo::z;
use foo::{A, B};
pub(crate) use foo::{C, D};
use foo0::{A, B};
