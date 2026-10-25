pub struct Public(pub u32);

pub fn exported() -> u32 {
    1
}

pub fn crate_value() -> u32 {
    2
}

pub mod a {
    pub fn first() -> u32 {
        4
    }

    pub fn second() -> u32 {
        8
    }
}

pub mod z {
    pub fn z_value() -> u32 {
        16
    }
}
