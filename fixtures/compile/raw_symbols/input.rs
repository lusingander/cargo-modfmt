use r#std::num::NonZeroU8;
use crate::r#type::r#match as r#async;
use crate::r#type::r#gen as Alias;
use crate::r#type::plain as r#loop;
use crate::r#type::r#plain as Second;
mod r#type;
mod alpha;

fn main() {
    let total = r#async() + Alias() + r#loop() + Second() + alpha::value()
        + u32::from(NonZeroU8::new(32).unwrap().get());
    println!("{total}");
}
