use foo::{
    self, self as One, self as Two, A as Alias,
    nested::{self as Ns, B, C},
};

fn explicit_self() {}

use crate::{self as Root, Item};

fn crate_root() {}

mod child {
    use self::super::{self as AlsoParent, Other};
    use self::{self as Here, local::A};
    use super::{self as Parent, Item};
}
