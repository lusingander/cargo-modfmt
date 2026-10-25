use foo::Before;

fn boundary() {}

use foo::After;

mod outer {
    use crate::model::{A, B};

    mod child { use super::super::model::{C, D}; }
}

/// A configured module.
#[cfg(any())]
mod configured {
    use self::model::{A, B};
}

#[custom]
mod untouched {
    use foo::B;
    use foo::A;
    mod child { use foo::D; use foo::C; }
}

mod header {
    #![allow(dead_code)]
    use foo::{A, B};

    fn boundary() {}

    use foo::{C, D};
}

fn example( ) {
    use foo::B;
    use foo::A;
    if true { use foo::D; use foo::C; }
    let text = "use foo::B; use foo::A;";
}

macro_rules! untouched {
    () => { use foo::B; use foo::A; };
}

emit! { use foo::B; use foo::A; }
