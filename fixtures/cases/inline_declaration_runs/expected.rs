mod outer {
    mod a;
    mod z;

    use crate::Thing;

    struct Boundary;

    mod a;
    mod r#type;

    pub mod z;

    fn boundary() {}

    mod a;
    mod b;

    pub mod z;

    mod child {
        mod b;
        mod d;
    }
}

struct Thing;
