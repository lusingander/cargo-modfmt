use foo::{
    Item, Item as Alpha, Item as Zed, Item as _,
    a::Thing as Z,
    a::{A as Y, B as Z},
    z::Thing as A,
};

fn sibling_order() {}

pub use foo::Item as Z;
use foo::Item as A;
pub(crate) use foo::Item as _;

fn repeated_aliases() {}

use foo::{Item as B, Item as A, Item as B};
