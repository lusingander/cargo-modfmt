mod outer {
    pub(crate) use crate::model::{CrateA, CrateZ, InCrateA, InCrateZ};
    pub(in crate::outer) use crate::model::{OuterA, OuterZ};
    pub(super) use crate::model::{ParentA, ParentZ};
    use crate::model::{PrivateA, PrivateZ};
    pub use crate::model::{PubA, PubZ};
    pub(self) use crate::model::{SelfA, SelfZ};
}
