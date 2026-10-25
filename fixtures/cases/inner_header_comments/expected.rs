// License stays before the inner header.
#![allow(dead_code)]
//! Crate documentation.
mod z; mod a;
fn root_boundary() {}
mod b;
mod d;

mod before {
    // Before the header.
    #![allow(dead_code)]
    //! Details.
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod between {
    #![allow(dead_code)]
    /* Between headers. */
    //! Details.
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod after {
    #![allow(dead_code)]
    //! Details.

    // Before the declarations.

    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod inside {
    #![allow(dead_code)]
    mod z /* Name stays attached. */; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod trailing {
    #![allow(dead_code)]
    mod z; mod a; // Trailing comment.
    fn boundary() {}
    mod b;
    mod d;
}

mod following {
    #![allow(dead_code)]
    mod z; mod a;

    // After the declarations.

    fn boundary() {}
    mod b;
    mod d;
}

mod use_attribute {
    #![allow(dead_code)]
    #[allow(unused_imports)]
    use std::path::Path;
    use std::collections::BTreeMap;
    fn boundary() {}
    mod b;
    mod d;
}

mod mod_attribute {
    #![allow(dead_code)]
    #[cfg(test)]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod use_doc {
    //! Module docs.
    /// Import documentation.
    use std::path::Path;
    use std::collections::BTreeMap;
    fn boundary() {}
    mod b;
    mod d;
}

mod mod_doc {
    //! Module docs.
    /** Declaration documentation. */
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod unsupported_use {
    #![allow(dead_code)]
    use {std::path::Path, std::collections::BTreeMap};
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod attribute_comments {
    #![allow(/* Lint explanation. */ dead_code)]
    #![doc = "Text with // and /* markers */."]
    mod a;
    mod z;
    /// Function documentation.
    #[cfg(/* Condition explanation. */ test)]
    fn boundary() { /* Body comment. */ }
    mod b;
    mod d;
}

#[allow(dead_code)]
mod outer_lint {
    #![allow(dead_code)]
    mod z; mod a;
    mod child { mod y; mod x; }
}
