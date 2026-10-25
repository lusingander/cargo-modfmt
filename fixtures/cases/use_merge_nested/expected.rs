use foo::{
    a::{
        A, Z,
        nested::{B, C, D},
    },
    b::{A, B},
    z::Only,
};

fn boundary() {}

use foo::deep::module::{A, Z};
