pub ( crate ) use   ::foo :: z :: Z as r#type  ;
pub(crate) use ::foo::a::{self as Root, *};
pub use ::foo::B;
pub use ::foo::A;
use foo::Item as Local;

fn visibility() {}

use ::foo :: r#Item
    as r#type  ;

fn simple_layout() {}

pub use   ::foo :: { *, r#type as r#match, A, self as Root, }  ;
