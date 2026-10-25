pub use ::foo::{A, B};
pub(crate) use ::foo::{
    a::{self as Root, *},
    z::Z as r#type,
};
use foo::Item as Local;

fn visibility() {}

use ::foo::r#Item as r#type;

fn simple_layout() {}

pub use ::foo::{self as Root, A, r#type as r#match, *};
