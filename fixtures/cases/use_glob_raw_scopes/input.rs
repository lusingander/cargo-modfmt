use ::foo::B;
mod r#type;
use ::foo::A;
use r#std::io;
mod alpha;
use crate::r#model::*;
use crate::r#model::Item;

#[cfg(any())]
mod r#async {
    use super::r#model::Z;
    use super::r#model::{*, self as Model, A};
}

#[macro_use]
mod blocked {
    use ::foo::Z;
    use ::foo::{*, A};
}

fn untouched() {
    use ::foo::Z;
    use foo::{r#type, *};
    let text = "use ::foo::{Z, r#type, *};";
    macro_rules! keep { () => { use ::foo::Z; use foo::{r#type, *}; } }
}

use ::foo::Z;
use ::foo::A;
