use std::path::Path;
use std::collections::BTreeMap;

pub fn run(value: usize) -> usize {
    value + 13 + Path::new("").as_os_str().len() + BTreeMap::<u8, u8>::new().len()
}
