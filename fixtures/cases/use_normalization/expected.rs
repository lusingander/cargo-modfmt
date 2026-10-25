pub(crate) use foo::{Item, Other};

fn boundary() {}

use bar::nested::{A, B};

fn other() {}

use baz::*;

fn raw_alias() {}

pub(self) use raw::r#item;
pub(super) use raw::r#type;

fn duplicate_after_normalization() {}

use qux::Item;
use qux::Item;
use qux::Other;
