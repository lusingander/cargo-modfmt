use foo::Z;
use foo::r#Item as r#Alias;
use foo::Item as Alias;

fn duplicate_logical_names() {}

use foo::a::A;
use foo::r#a;

fn ordinary_prefix_with_different_spelling() {}

use r#foo::B;
use foo::A;
use r#foo::A;
use foo::B;

fn raw_and_plain_roots() {}

use foo::r#a::*;
use foo::a::*;
use foo::Z;
