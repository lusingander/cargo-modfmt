use foo::r#Item as r#Alias;
use foo::Item as Alias;
use foo::Z;

fn duplicate_logical_names() {}

use foo::r#a;
use foo::a::A;

fn ordinary_prefix_with_different_spelling() {}

use r#foo::{A, B};
use foo::{A, B};

fn raw_and_plain_roots() {}

use foo::Z;
use foo::r#a::*;
use foo::a::*;
