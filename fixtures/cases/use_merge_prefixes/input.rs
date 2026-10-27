use crate::alpha::{Alpha, Beta, Delta, Gamma};
use crate::beta::Item;
use crate::gamma::Store;
use crate::nested;
use crate::nested::child;
use crate::omega::{Command, State};
use vendor::Handle;
use vendor::channels::packet::{self, Header, Packet, PacketKind};
use vendor::geometry::{Bounds, Point};
use std::path;

fn crate_prefix() {}

use foo::a::b::D;
use foo::a::b::C;
use foo::a::b;
use foo::a::B;
use foo::a;

fn nested_prefixes() {}

use foo::a as _;
use foo::a as Second;
use foo::a::{self as TypeOnly, B, *};
use foo::a::nested as Module;
use foo::a::nested::{self as Nested, Child, *};
use foo::a as First;
use foo::Z;

fn aliases_self_globs() {}

mod scoped {
    use self::a::B;
    use self::a;
    pub use super::a::B;
    pub use super::a;
    use super::super::a::B;
    use super::super::a;
    use ::foo::a::B;
    use ::foo::a;
}
