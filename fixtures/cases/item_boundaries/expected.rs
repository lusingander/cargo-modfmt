mod alpha;
mod zebra;

fn example(  ) {
    use std::io;
    let source = r#"mod fake; // use fake::Item;"#;
}

mod beta;
mod delta;

mod nested {
    mod a;
    mod z;
}

mod x;
mod y;

macro_rules! declare {
    () => {
        mod generated_z;
        mod generated_a;
    };
}

declare! {
    use fake::Item;
    mod generated;
}
