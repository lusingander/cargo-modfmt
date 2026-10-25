use foo::nested::C;
use foo::{self as Two, nested::{self as Ns, B}};
use foo::self as One;
use foo::{self, A as Alias};

fn explicit_self() {}

use crate as Root;
use crate::Item;

fn crate_root() {}

mod child {
    use super::Item;
    use self as Here;
    use super as Parent;
    use self::local::A;
    use self::super as AlsoParent;
    use self::super::Other;
}
