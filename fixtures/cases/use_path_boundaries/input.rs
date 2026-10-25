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

use z::Z;
use foo::self;
use a::A;

fn keyword_leaf() {}

use z::Z;
use foo::super::Thing;
use a::A;

fn keyword_after_name() {}

use z::Z;
use crate::super::Thing;
use a::A;

fn super_after_crate() {}

use z::Z;
use r#std::io;
use a::A;

fn raw_root() {}

use z::Z;
use foo::Thing as _;
use a::A;

fn underscore_alias() {}

use z::Z;
use self::super::super::Ancestor;
use a::A;
