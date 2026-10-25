use foo::a::{self as Local, B};
pub use foo::{
    a::{self as Public},
    z::Z as Exported,
};
pub(crate) use foo::{
    a::{self as Restricted, A},
    z::{Z as Last, Z as Separate},
};

fn visibility() {}

use a::Thing as Other;
use z::Item as Alias;

fn singleton_alias() {}

pub use foo::{self as Root, A as First, z::Z as Last};
