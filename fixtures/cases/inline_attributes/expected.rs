mod a;
mod z;

/// Module documentation.
#[cfg(test)]
mod tests {
    mod a;
    mod z;

    mod child {
        mod x;
        mod y;
    }
}

mod b;
mod d;

mod plain {
    #[path = "other.rs"]
    mod z;
    mod a;

    fn boundary() {}

    mod b;
    mod d;
}

/// Documented module.
mod documented {
    mod a;
    mod z;
}

#[cfg(test)]
mod cfg_only {
    mod a;
    mod z;
}

#[doc = "Module documentation."]
#[doc(hidden)]
mod doc_attribute {
    mod a;
    mod z;
}

#[custom]
mod custom_attribute {
    mod z;
    mod a;

    mod child {
        mod y;
        mod x;
    }
}

#[cfg(test)]
#[macro_use]
mod mixed_attributes {
    mod z;
    mod a;
}

#[cfg_attr(test, custom)]
mod conditional_attributes {
    mod z;
    mod a;
}

/** Block module documentation. */
mod block_documented {
    mod a;
    mod z;
}

#[custom::doc]
mod qualified_doc {
    mod z;
    mod a;
}
