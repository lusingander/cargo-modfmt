#![allow(dead_code)]
mod a;
mod z;

use std::io;

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

mod a;
mod z;

use std::io;

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

    mod a;
    mod z;

    use std::io;
}
