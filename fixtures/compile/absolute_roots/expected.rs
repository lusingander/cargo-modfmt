extern crate vendor;

mod model;

use ::std::{mem::size_of, num::NonZeroU8};

mod outer {
    mod vendor {
        pub fn a() -> u32 { 1 }
        pub fn z() -> u32 { 2 }
    }

    use ::vendor::{a as absolute_a, z as absolute_z};
    use vendor::{a as relative_a, z as relative_z};

    pub fn total() -> u32 {
        assert_eq!(self::vendor::a() + self::vendor::z(), 3);
        relative_a() + relative_z() + absolute_a() + absolute_z()
    }
}

fn main() {
    assert_eq!(size_of::<u8>(), 1);
    assert_eq!(NonZeroU8::new(1).unwrap().get(), 1);
    assert_eq!(model::marker(), 4);
    println!("{}", outer::total());
}
