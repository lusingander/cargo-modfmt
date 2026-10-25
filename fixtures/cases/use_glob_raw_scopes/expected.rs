mod alpha;
mod r#type;

use r#std::io;

use ::foo::{A, B};

use crate::r#model::{Item, *};

#[cfg(any())]
mod r#async {
    use super::r#model::{self as Model, A, Z, *};
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

use ::foo::{A, Z};
