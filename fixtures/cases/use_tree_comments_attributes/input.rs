use foo::{B, /* keep */ A};

fn block_comment() {}

mod z;
use foo::{z::Z, a::{B, // keep
    A}};
mod a;

fn nested_line_comment() {}

// A nearby explanation.

use foo::{B, A};

fn preceding_comment() {}

use foo::{B, A}; // keep

fn trailing_comment() {}

mod z;
#[cfg(any())]
use foo::{B, A};
mod a;

fn use_attribute() {}

/// An import description.
use foo::{B, A};

fn use_documentation() {}

use foo::{B, A};

/// A function description.
#[allow(dead_code)]
fn adjacent_documented_item() {}

use foo::{z::Z, a::{B, A}};
