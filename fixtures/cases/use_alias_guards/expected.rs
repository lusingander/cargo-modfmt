use foo::Z as Last;
use foo::{self, /* keep */ A};

fn comment() {}

use foo::Z;
#[cfg(any())]
use foo::A as Alias;

fn attribute() {}

use bar::Trait as r#type;
use foo::{self, A};

fn raw_alias() {}

use bar::*;
use foo::{self, A};

fn glob() {}

use ::bar::B;
use foo::A as Alias;

fn absolute() {}

use foo::{self, A};
use {bar::B, bar::A};

fn root_group() {}

use foo::{self, A};
use bar::{};

fn empty_group() {}

use foo::{self::a::A, Z as Last};
use bar::B;

fn nested_qualifier() {}

use z::Z as Last;
use self;
use a::A;

fn bare_qualifier() {}

use foo::Z as Last;
use crate::{self};
use foo::A;

fn bare_crate_self() {}

use foo::Z as Last;
use self::super::{self};
use foo::A;

fn bare_ancestor_self() {}

use foo::{A, Z as Last};

/// A separate item.
fn documented_item() {}
