mod r#alpha;
mod beta;
mod zeta;

pub mod gamma;
pub mod r#match;

use ::std::fmt;
use r#core::fmt;
use r#std::io;

use r#foo::{A, r#Z, r#alpha, r#type, r#type as Alpha, r#type as r#Zed, r#type as _};

fn names_without_raw_prefix() {}

pub use r#foo::Item as Raw;
use foo::Item as Plain;
