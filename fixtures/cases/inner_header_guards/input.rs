#![allow(dead_code)]
#![cfg_attr(test, doc(hidden))]
mod z; mod a;
fn root_boundary() {}
mod d; mod b;

mod cfg_attr {
    //! Module documentation.
    #![allow(dead_code)]
    #![cfg_attr(test, allow(dead_code))]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod custom {
    //! Module documentation.
    #![allow(dead_code)]
    #![custom]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod qualified_doc {
    //! Module documentation.
    #![allow(dead_code)]
    #![custom::doc = "Details."]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod qualified_lint {
    //! Module documentation.
    #![allow(dead_code)]
    #![custom::allow(dead_code)]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod absolute_doc {
    //! Module documentation.
    #![allow(dead_code)]
    #![::doc = "Details."]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod absolute_lint {
    //! Module documentation.
    #![allow(dead_code)]
    #![::allow(dead_code)]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod macro_use {
    //! Module documentation.
    #![allow(dead_code)]
    #![macro_use]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod tool {
    //! Module documentation.
    #![allow(dead_code)]
    #![rustfmt::skip]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod crate_setting {
    //! Module documentation.
    #![allow(dead_code)]
    #![recursion_limit = "128"]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod lint_atom {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod lint_value {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow = "dead_code"]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod lint_brackets {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow[dead_code]]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod lint_braces {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow{dead_code}]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod unsafe_doc {
    //! Module documentation.
    #![allow(dead_code)]
    #![unsafe(doc(hidden))]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
    mod child {
        mod y; mod x;
    }
}

mod absolute_cfg {
    #![::cfg(all())]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
}

mod qualified_cfg {
    #![custom::cfg(all())]
    mod z; mod a;
    fn boundary() {}
    mod d; mod b;
}
