#[cfg(feature = "example")]
mod zebra;
mod alpha;

fn first_boundary() {}

/// Module description.
mod y;
mod x;

struct Marker;

#[path = "alternate.rs"]
mod b;
mod a;

enum Placeholder {}

#[macro_use]
mod d;
mod c;
