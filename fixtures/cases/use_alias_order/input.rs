use foo::{z::Thing as A, Item as Zed, Item as _, Item, a::Thing as Z, Item as Alpha, a::{B as Z, A as Y}};

fn sibling_order() {}

pub use foo::Item as Z;
use foo :: Item as A;
pub(crate) use foo::Item as _;

fn repeated_aliases() {}

use foo::{Item as B, Item as A, Item as B};
