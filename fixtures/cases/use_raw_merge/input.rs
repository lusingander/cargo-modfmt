use foo::r#beta::B;
use foo::alpha::A;
use foo::r#beta::A as r#type;
use foo::r#Item as r#Alias;
use foo::Item as Other;

fn same_spelling_prefix() {}

use foo::r#a::B;
use foo::a::A;
use foo::a::Z;
use foo::r#a::C;
