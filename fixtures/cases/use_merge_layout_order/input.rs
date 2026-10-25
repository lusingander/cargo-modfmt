    pub ( crate ) use   crate :: z :: Z  ;
pub(crate) use crate::a::A;
use crate :: single :: Item;

fn boundary( ) { let untouched = "use foo::B; use foo::A;"; }

pub use foo::deep::Z;
use foo::a::A;
pub use foo::deep::A;

mod compact { use foo::B; use foo::A; }
