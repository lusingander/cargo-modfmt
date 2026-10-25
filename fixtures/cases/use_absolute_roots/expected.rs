use ::core::fmt;
use ::std::io;

use ::foo::{A, B};
use foo::{A, B};

fn separate_roots() {}

use ::foo::{A, B};
use bar::{A, B};

fn path_before_anchor() {}

pub use ::foo::Item as Absolute;
use foo::Item as Relative;
