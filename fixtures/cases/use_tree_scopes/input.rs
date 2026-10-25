use foo::{Z, A};

mod outer { use crate::{z::Z, a::A};
    mod child {
        use super::super::{z::Z, a::A};
    }
}

#[cfg(any())]
#[doc = "A module"]
mod configured {
    use self::{z::Z, a::A};
}

#[custom]
mod untouched {
    use foo::{Z, A};
    mod child { use foo::{Z, A}; }
}

mod header {
    #![allow(dead_code)]
    use foo::{Z, A};

    fn boundary() {}

    use foo::{Z, A};

    mod child { use foo::{Z, A}; }
}

mod documented_header {
    //! A scope header.
    use foo::{Z, A};
}

fn example( ) {
    use foo::{Z, A};
    if true { use foo::{Z, A}; }
    mod nested { use foo::{Z, A}; }
    let text = "use foo::{Z, A};";
}

macro_rules! untouched {
    () => { use foo::{Z, A}; };
}

emit! { use foo::{Z, A}; }
