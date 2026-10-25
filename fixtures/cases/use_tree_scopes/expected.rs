use foo::{A, Z};

mod outer { use crate::{a::A, z::Z};
    mod child {
        use super::super::{a::A, z::Z};
    }
}

#[cfg(any())]
#[doc = "A module"]
mod configured {
    use self::{a::A, z::Z};
}

#[custom]
mod untouched {
    use foo::{Z, A};
    mod child { use foo::{Z, A}; }
}

mod header {
    #![allow(dead_code)]
    use foo::{A, Z};

    fn boundary() {}

    use foo::{A, Z};

    mod child { use foo::{A, Z}; }
}

mod documented_header {
    //! A scope header.
    use foo::{A, Z};
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
