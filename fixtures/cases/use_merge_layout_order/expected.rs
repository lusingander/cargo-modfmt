    use crate::single::Item;
    pub(crate) use crate::{a::A, z::Z};

fn boundary( ) { let untouched = "use foo::B; use foo::A;"; }

use foo::a::A;
pub use foo::deep::{A, Z};

mod compact { use foo::{A, B}; }
