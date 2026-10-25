use foo::B;
use foo::A; // keep

fn trailing_comment() {}

use foo::B;
use foo::{a::{C, /* keep */ A}};

fn nested_comment() {}

// A nearby explanation.

use foo::B;
use foo::A;

fn preceding_comment() {}

use foo::B;
#[cfg(any())]
use foo::A;

fn attribute() {}

use foo::B;
use foo::A;
use bar::{self, Item};

fn type_only_neighbor() {}

use foo::B;
use foo::A;
use bar::{};

fn empty_group() {}

use foo::B;
use foo::A;

/// A function description.
fn documented_item() {}

use foo::D;
use foo::C;
