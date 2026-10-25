#![allow(dead_code)]
mod a;
mod z;

fn root_boundary() {}

mod b;
mod d;

#[cfg(test)]
mod attributed {
    #![allow(dead_code)]
    mod a;
    mod z;

    fn boundary() {}

    mod b;
    mod d;

    mod child {
        mod x;
        mod y;
    }
}

mod documented {
    //! Module documentation.
    mod a;
    mod z;

    fn boundary() {}

    mod b;
    mod d;
}

mod first_child {
    #![cfg(any())]

    mod child {
        #![allow(dead_code)]
        use std::{collections::BTreeMap, path::Path};
    }

    mod a;
    mod z;
}

mod first_function {
    #![custom]
    fn boundary() {}
    mod a;
    mod z;
}

mod same_line { #![allow(dead_code)] use std::{collections::BTreeMap, path::Path}; }

mod empty {
    //! No declarations.
    #![allow(dead_code)]
}
