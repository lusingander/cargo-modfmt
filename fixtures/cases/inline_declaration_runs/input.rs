mod outer {
    mod z;
    use crate::Thing;
    mod a;

    struct Boundary;

    mod r#type;
    pub mod z;
    mod a;

    fn boundary() {}

    pub mod z;
    mod b;
    mod a;

    mod child {
        mod d;
        mod b;
    }
}

struct Thing;
