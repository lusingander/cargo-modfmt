use foo::{
    r#Item as r#Alias, Item as Other,
    alpha::A,
    r#beta::{A as r#type, B},
};

fn same_spelling_prefix() {}

use foo::{
    a::{A, Z},
    r#a::{B, C},
};
