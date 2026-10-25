pub ( crate ) use   foo :: z :: Z as Last  ;
use foo::a::{self as Local, B};
pub(crate) use foo::a::{self as Restricted, A};
pub use foo::a::{self as Public};
pub use foo::z::Z as Exported;
pub(in crate) use foo::z::Z as Separate;

fn visibility() {}

use z :: Item
    as Alias  ;
use a::Thing as Other;

fn singleton_alias() {}

pub use   foo :: { z::Z as Last, self as Root, A as First, }  ;
