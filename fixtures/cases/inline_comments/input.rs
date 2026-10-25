mod zebra;
mod alpha;

// Module description.
mod /* header */ outer {
    mod z;
    mod a;

    fn boundary() { /* body comment */ }

    mod d;
    mod b; // Declaration comment.

    mod child {
        mod y;
        mod x;
    }

    // Scope comment.
    mod w;
    mod v;

    fn second_boundary() {}

    mod f;
    mod e;
}

mod before_z;
mod before_a;

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
    mod z;
    mod a;
}
// Outside scope.
mod after_z;
mod after_a;
