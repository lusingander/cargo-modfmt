mod zebra;
mod alpha;

fn example(  ) {
    use std::io;
    let source = r#"mod fake; // use fake::Item;"#;
}

mod delta;
mod beta;

mod nested {
    mod z;
    mod a;
}

mod y;
mod x;

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
