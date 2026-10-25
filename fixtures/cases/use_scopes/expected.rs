mod a;
mod z;

use std::fmt::Debug;

use crate::z::Z;

/// Test module documentation.
#[cfg(test)]
mod tests {
    mod a;
    mod z;

    use super::Z;
    use crate::a::A;

    mod nested {
        use self::Item;
        use super::super::Thing;
        use crate::a::A;
    }

    fn example(  ) {
        use std::io;
        use std::fmt;
        let source = "use fake::Item; mod fake;";
    }

    const BLOCK: () = {
        use std::io;
        use std::fmt;
    };

    macro_rules! declare {
        () => {
            use std::io;
            use std::fmt;
        };
    }

    declare! {
        use std::io;
        use std::fmt;
    }
}

use std::io;

use crate::d::D;

#[custom]
mod untouched {
    use crate::z::Z;
    mod z;
    use std::io;
    mod a;
}
