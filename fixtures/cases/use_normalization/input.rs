pub(in crate) use foo::{Item as Item};
pub(crate) use foo::Other;

fn boundary() {}

use bar::{nested::{B, A}};

fn other() {}

use baz::{*};

fn raw_alias() {}

pub(in self) use raw::r#item as item;
pub(in super) use raw::r#type as r#type;

fn duplicate_after_normalization() {}

use qux::Item as Item;
use qux::Item;
use qux::Other;
