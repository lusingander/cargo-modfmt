pub struct Public(pub u32);

pub fn root_value() -> u32 {
    1
}

pub fn local_value() -> u32 {
    2
}

pub fn parent_value() -> u32 {
    4
}

pub fn crate_value() -> u32 {
    8
}

pub fn qualified_value() -> u32 {
    16
}

pub fn child_value() -> u32 {
    32
}
