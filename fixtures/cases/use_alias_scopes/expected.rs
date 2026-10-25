mod a;
mod z;

use std::path::Path as FsPath;

use foo::Item as External;

use crate::model::{self as Model, Item};

#[cfg(any())]
mod allowed {
    use super::model::{self as Model, A, Z as Last};

    fn untouched() { use foo::Z as Last; use foo::A; }
}

#[macro_use]
mod blocked {
    use foo::Z as Last;
    use foo::{self, A};
}

fn untouched() {
    use foo::Z as Last;
    use foo::{self, A};
    let text = "use foo::{Z as Last, self, A};";
    macro_rules! keep { () => { use foo::Z as Last; use foo::A; } }
}

use foo::{A, B as Bee};
