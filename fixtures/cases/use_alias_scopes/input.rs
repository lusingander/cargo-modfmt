use std::path::Path as FsPath;
use foo::Item as External;
mod z;
use crate::model::{self as Model};
mod a;
use crate::model::Item;

#[cfg(any())]
mod allowed {
    use super::model::Z as Last;
    use super::model::{self as Model, A};

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

use foo::B as Bee;
use foo::A;
