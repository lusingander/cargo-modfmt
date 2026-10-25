use foo::a::*;
use bar::Z;
use foo::{a::{*}, B};
use foo::C;
use bar::A;

fn duplicate_glob() {}

use foo::*;
use foo as Root;

fn ordinary_root() {}

use foo::a::*;
use foo::B;
use foo::a as Module;
