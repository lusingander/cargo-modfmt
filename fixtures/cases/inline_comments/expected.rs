mod zebra;
mod alpha;

// Module description.
mod /* header */ outer {
    mod a;
    mod z;

    fn boundary() { /* body comment */ }

    mod d;
    mod b; // Declaration comment.

    mod child {
        mod x;
        mod y;
    }

    // Scope comment.
    mod w;
    mod v;

    fn second_boundary() {}

    mod e;
    mod f;
}

mod before_a;
mod before_z;

mod inner_comments {
    // Body header.
    mod z;
    mod a;
}

mod tail_comments {
    mod z;
    mod a;
    // Body tail.
}

mod outer_comments {
    mod a;
    mod z;
}
// Outside scope.
mod after_z;
mod after_a;
