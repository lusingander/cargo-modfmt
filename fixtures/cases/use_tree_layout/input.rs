    pub(crate) use   foo :: { z :: Z , a :: { B , A, }, }  ;
use   foo :: plain;

fn boundary( ) { let untouched = "use foo::{B, A};"; }

pub   use   crate :: {
    z :: Z,
    a :: {B, A,},
}  ;

struct Boundary;

use foo::{A, B,};
