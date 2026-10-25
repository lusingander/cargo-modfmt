#![allow(dead_code)]
//! Crate docs.
mod z;
mod a;

mod child {
    /*! Module docs. */
    #![doc(hidden)]
    mod z;
    mod a;
}

mod mixed {
    #![allow(dead_code)]
    mod z;
    mod a;
    fn boundary() {}
    mod d; mod b;
}
