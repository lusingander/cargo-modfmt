    use foo::plain;
    pub(crate) use foo::{
        a::{A, B},
        z::Z,
    };

fn boundary( ) { let untouched = "use foo::{B, A};"; }

pub use crate::{
    a::{A, B},
    z::Z,
};

struct Boundary;

use foo::{A, B};
