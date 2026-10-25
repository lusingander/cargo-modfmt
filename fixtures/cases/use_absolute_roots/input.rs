use ::foo::B;
use foo::A;
use ::std::io;
use ::foo::A;
use foo::B;
use ::core::fmt;

fn separate_roots() {}

use ::foo::{B, A};
use bar::{B, A};

fn path_before_anchor() {}

pub use ::foo::Item as Absolute;
use foo::Item as Relative;
