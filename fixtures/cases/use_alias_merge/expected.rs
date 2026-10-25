use foo::{Item, Item as Alpha, Item as Zed, Other as _, Trait as _};

fn distinct_aliases() {}

use foo::{Item as Same, Other as Same};
