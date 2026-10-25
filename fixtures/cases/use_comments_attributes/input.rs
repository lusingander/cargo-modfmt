#![allow(dead_code)]
use std::io;
mod z;
mod a;

fn header_boundary() {}

mod z;
use std:: /* Import comment. */ io;
mod a;

fn comment_boundary() {}

#[cfg(test)]
use crate::Thing;
mod z;
mod a;

fn attribute_boundary() {}

/// Re-export documentation.
pub use crate::Thing;
mod z;
use std::io;
mod a;

fn documentation_boundary() {}

mod z;
use std::io;
mod a;

/// Function documentation.
#[cfg(test)]
fn item_boundary() {}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use crate::Thing;
    mod z;
    mod a;

    fn boundary() {}

    use std::io;
    mod z;
    mod a;
}
