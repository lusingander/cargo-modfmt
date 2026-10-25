mod z;
use {std::io, crate::Item};
mod a;

fn mixed_roots() {}

use {std::path::Path, std::io::Read};
use foo::{B, A};

fn same_category_roots() {}

use foo::{z::Z, {B, A}};
use bar::{B, A};

fn nested_anonymous_group() {}

use {};
use foo::{B, A};

fn empty_root() {}

use foo::{};
use bar::{B, A};

fn empty_group() {}

mod z;
use foo::{z::Z, a::{}, B};
mod a;

fn nested_empty_group() {}

use foo::{z::Z, a::{B, A}};
