#![allow(dead_code)]
//! Crate docs.
mod a;
mod z;

mod child {
    /*! Module docs. */
    #![doc(hidden)]
    mod a;
    mod z;
}

mod mixed {
    #![allow(dead_code)]
    mod z;
    mod a;
    fn boundary() {}
    mod b;
    mod d;
}
