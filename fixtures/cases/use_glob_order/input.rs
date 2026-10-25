use foo::{z::{Z, *}, *, self as Root, B, a::*, a::{C, self as Module, *, A}};

fn nested_order() {}

use foo :: a :: *  ;
