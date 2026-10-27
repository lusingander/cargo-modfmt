use std::path;

use vendor::{
    Handle,
    channels::packet::{self, Header, Packet, PacketKind},
    geometry::{Bounds, Point},
};

use crate::{
    alpha::{Alpha, Beta, Delta, Gamma},
    beta::Item,
    gamma::Store,
    nested,
    nested::child,
    omega::{Command, State},
};

fn crate_prefix() {}

use foo::{
    a,
    a::{
        B, b,
        b::{C, D},
    },
};

fn nested_prefixes() {}

use foo::{
    Z, a as First, a as Second, a as _,
    a::{
        self as TypeOnly, B, nested as Module,
        nested::{self as Nested, Child, *},
        *,
    },
};

fn aliases_self_globs() {}

mod scoped {
    use ::foo::{a, a::B};

    use self::{a, a::B};
    use super::super::{a, a::B};
    pub use super::{a, a::B};
}
