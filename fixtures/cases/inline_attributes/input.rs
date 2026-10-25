mod z;
mod a;

/// Module documentation.
#[cfg(test)]
mod tests {
    mod z;
    mod a;

    mod child {
        mod y;
        mod x;
    }
}

mod d;
mod b;

mod plain {
    #[path = "other.rs"]
    mod z;
    mod a;

    fn boundary() {}

    mod d;
    mod b;
}

/// Documented module.
mod documented {
    mod z;
    mod a;
}

#[cfg(test)]
mod cfg_only {
    mod z;
    mod a;
}

#[doc = "Module documentation."]
#[doc(hidden)]
mod doc_attribute {
    mod z;
    mod a;
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
    mod z;
    mod a;
}

#[custom::doc]
mod qualified_doc {
    mod z;
    mod a;
}
