mod alpha;
mod zebra;

pub mod outer {
    mod beta;
    mod delta;

    pub(in crate::outer) mod alpha;
    pub(super) mod zebra;

    mod child {
        mod x;
        mod y;

        mod grandchild {
            mod a;
            mod b;

            pub mod z;
        }

        mod c;
        mod d;
    }

    mod epsilon;
    mod gamma;

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

mod xray;
mod yankee;
