pub struct Pair(pub u32);

pub trait Score {
    fn score(&self) -> u32;
}

impl Score for u32 {
    fn score(&self) -> u32 {
        self * 3
    }
}

pub mod branch {
    pub fn value() -> u32 {
        32
    }

    pub fn child() -> u32 {
        64
    }
}
