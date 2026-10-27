pub mod branch {
    pub fn child() -> u32 {
        2
    }
}

pub fn branch() -> u32 {
    1
}

#[macro_export]
macro_rules! branch_macro {
    () => { 4_u32 };
}

pub use crate::branch_macro as branch;

pub mod fallback {
    pub fn branch() -> u32 {
        64
    }
}
