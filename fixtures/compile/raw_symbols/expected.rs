mod alpha;
mod r#type;

use r#std::num::NonZeroU8;

use crate::r#type::{r#gen as Alias, r#match as r#async, r#plain as Second, plain as r#loop};

fn main() {
    let total = r#async() + Alias() + r#loop() + Second() + alpha::value()
        + u32::from(NonZeroU8::new(32).unwrap().get());
    println!("{total}");
}
