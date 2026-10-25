use ::foo::Z;
use ::foo::{r#type, /* keep */ *};

fn comment() {}

mod z;
#[cfg(any())]
mod r#type;
mod a;

fn attribute() {}

use foo::{r#type, *};
use {::bar::B, bar::A};

fn root_group() {}

use ::foo::{Z, r#type};
use bar::{};

fn empty_group() {}

use foo::{Z, self::a::A, *};
use ::bar::B;

fn nested_qualifier() {}

use foo::{r#type, *};
use self;

fn bare_qualifier() {}

use foo::r#type;
use *;

fn pathless_glob() {}

use foo::r#type;
use ::*;

fn pathless_absolute_glob() {}

use foo::{Z, ::bar::B, *};
use bar::A;

fn nested_absolute() {}

use ::super::Item;
use foo::{Z, r#type};

fn absolute_qualifier() {}

use foo::{Z, ::*, *};
use bar::A;

fn nested_pathless_absolute_glob() {}

use ::foo::{A, Z};

/// A separate item.
fn documented_item() {}
