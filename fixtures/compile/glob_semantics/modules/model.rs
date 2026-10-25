pub mod traits {
    pub trait Score {
        fn score(&self) -> u32;
    }

    impl Score for u32 {
        fn score(&self) -> u32 { self * 3 }
    }
}

pub mod glob_side {
    pub use super::traits::Score as _;

    pub struct Pair(pub u32);

    pub fn value() -> u32 { 1 }

    pub mod nested {
        pub fn nested_value() -> u32 { 64 }
    }
}

pub mod explicit {
    pub fn value() -> u32 { 32 }
}
