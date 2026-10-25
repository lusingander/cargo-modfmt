use crate::z::Z;
mod z;
use std::fmt::Debug;
mod a;

/// Test module documentation.
#[cfg(test)]
mod tests {
    use super::Z;
    mod z;
    use crate::a::A;
    mod a;

    mod nested {
        use super::super::Thing;
        use self::Item;
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

use crate::d::D;
use std::io;

#[custom]
mod untouched {
    use crate::z::Z;
    mod z;
    use std::io;
    mod a;
}
