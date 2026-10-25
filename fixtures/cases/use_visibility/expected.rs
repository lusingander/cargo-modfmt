mod outer {
    pub(in crate::outer) use crate::a::A;
    pub(crate) use crate::a::A;
    pub(self) use crate::b::B;
    pub(super) use crate::c::C;
    use crate::{a::A, m::M};
    pub use crate::{a::A, z::Z};
}
