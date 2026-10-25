use foo::B as Bee;
use foo::{self, A};

fn between_runs() {}

use foo::C as See;
use foo::nested::{
    self,
    A,
};

fn after_nested_mixed() {}

use foo::Z as Last;
use foo::{self, A};
