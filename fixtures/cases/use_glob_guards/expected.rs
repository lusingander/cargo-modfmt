use bar::{A, Z};
use foo::C;
use foo::a::*;
use foo::{B, a::*};

fn duplicate_glob() {}

use foo as Root;
use foo::*;

fn ordinary_root() {}

use foo::B;
use foo::a as Module;
use foo::a::*;
