use z::Z;
use crate;
use a::A;

fn bare_crate() {}

use z::Z;
use self;
use a::A;

fn bare_self() {}

use z::Z;
use super;
use a::A;

fn bare_super() {}

use a::A;
use foo::{self};
use z::Z;

fn keyword_leaf() {}

use z::Z;
use foo::super::Thing;
use a::A;

fn keyword_after_name() {}

use z::Z;
use crate::super::Thing;
use a::A;

fn super_after_crate() {}

use r#std::io;

use a::A;
use z::Z;

fn raw_root() {}

use a::A;
use foo::Thing as _;
use z::Z;

fn underscore_alias() {}

use a::A;
use z::Z;

use self::super::super::Ancestor;
