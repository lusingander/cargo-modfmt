#![allow(dead_code, clippy::unused_self, reason = "Keep the header.")]
#![warn(unused_imports)]
#![deny(unused_variables)]
#![forbid(unsafe_code)]
#![expect(dead_code)]
#![cfg(all())]
mod a;
mod z;

pub mod public;

use std::{collections::BTreeMap, path::Path};

use crate::model::{read, write};
fn boundary() {}
mod b;
mod d;
