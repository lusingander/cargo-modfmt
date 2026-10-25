mod zebra;
mod alpha;

pub mod outer {
    pub(super) mod zebra;
    mod delta;
    mod beta;
    pub(in crate::outer) mod alpha;

    mod child {
        mod y;
        mod x;

        mod grandchild {
            pub mod z;
            mod b;
            mod a;
        }

        mod d;
        mod c;
    }

    mod gamma;
    mod epsilon;

    fn example(  ) {
        mod untouched {
            mod z;
            mod a;
        }
        let source = "mod fake; use fake::Item;";
    }

    const BLOCK: () = {
        mod untouched {
            mod z;
            mod a;
        }
    };

    macro_rules! declare {
        () => {
            mod untouched {
                mod z;
                mod a;
            }
        };
    }

    declare! {
        mod untouched {
            mod z;
            mod a;
        }
    }
}

mod yankee;
mod xray;
