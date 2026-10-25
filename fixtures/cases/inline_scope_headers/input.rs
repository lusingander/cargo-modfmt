#![allow(dead_code)]
mod z; mod a;

fn root_boundary() {}

mod d;
mod b;

#[cfg(test)]
mod attributed {
    #![allow(dead_code)]
    mod z; mod a;

    fn boundary() {}

    mod d;
    mod b;

    mod child {
        mod y;
        mod x;
    }
}

mod documented {
    //! Module documentation.
    mod z; mod a;

    fn boundary() {}

    mod d;
    mod b;
}

mod first_child {
    #![cfg(any())]

    mod child {
        #![allow(dead_code)]
        use std::path::Path;
        use std::collections::BTreeMap;
    }

    mod z;
    mod a;
}

mod first_function {
    #![custom]
    fn boundary() {}
    mod z; mod a;
}

mod same_line { #![allow(dead_code)] use std::path::Path; use std::collections::BTreeMap; }

mod empty {
    //! No declarations.
    #![allow(dead_code)]
}
