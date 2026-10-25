#![allow(dead_code)]
#![cfg_attr(test, doc(hidden))]
mod z; mod a;
fn root_boundary() {}
mod b;
mod d;

mod cfg_attr {
    //! Module documentation.
    #![allow(dead_code)]
    #![cfg_attr(test, allow(dead_code))]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod custom {
    //! Module documentation.
    #![allow(dead_code)]
    #![custom]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod qualified_doc {
    //! Module documentation.
    #![allow(dead_code)]
    #![custom::doc = "Details."]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod qualified_lint {
    //! Module documentation.
    #![allow(dead_code)]
    #![custom::allow(dead_code)]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod absolute_doc {
    //! Module documentation.
    #![allow(dead_code)]
    #![::doc = "Details."]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod absolute_lint {
    //! Module documentation.
    #![allow(dead_code)]
    #![::allow(dead_code)]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod macro_use {
    //! Module documentation.
    #![allow(dead_code)]
    #![macro_use]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod tool {
    //! Module documentation.
    #![allow(dead_code)]
    #![rustfmt::skip]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod crate_setting {
    //! Module documentation.
    #![allow(dead_code)]
    #![recursion_limit = "128"]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod lint_atom {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod lint_value {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow = "dead_code"]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod lint_brackets {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow[dead_code]]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod lint_braces {
    //! Module documentation.
    #![allow(dead_code)]
    #![allow{dead_code}]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod unsafe_doc {
    //! Module documentation.
    #![allow(dead_code)]
    #![unsafe(doc(hidden))]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
    mod child {
        mod x;
        mod y;
    }
}

mod absolute_cfg {
    #![::cfg(all())]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}

mod qualified_cfg {
    #![custom::cfg(all())]
    mod z; mod a;
    fn boundary() {}
    mod b;
    mod d;
}
