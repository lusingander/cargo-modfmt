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

use bar::{self, Item};
use foo::{A, B};

fn type_only_neighbor() {}

use foo::B;
use foo::A;
use bar::{};

fn empty_group() {}

use foo::{A, B};

/// A function description.
fn documented_item() {}

use foo::{C, D};
