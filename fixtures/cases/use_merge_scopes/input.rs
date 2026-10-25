use foo::Before;

fn boundary() {}

use foo::After;

mod outer {
    use crate::model::B;
    use crate::model::A;

    mod child { use super::super::model::D; use super::super::model::C; }
}

/// A configured module.
#[cfg(any())]
mod configured {
    use self::model::B;
    use self::model::A;
}

#[custom]
mod untouched {
    use foo::B;
    use foo::A;
    mod child { use foo::D; use foo::C; }
}

mod header {
    #![allow(dead_code)]
    use foo::B;
    use foo::A;

    fn boundary() {}

    use foo::D;
    use foo::C;
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
