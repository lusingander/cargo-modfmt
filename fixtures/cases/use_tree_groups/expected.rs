mod a;
mod b;

pub(crate) mod a;
pub mod z;

use alloc::{string::String, vec::Vec};
use core::{cmp::Ordering, option::Option};
use std::{
    io::{Read, Write},
    path::Path,
};

use alpha::Thing;
use crate_utils::{A, B};
use stdish::{A, B};
use zed::{A, B};

use self::{a::A, z::Z};
use super::super::{a::A, z::Z};
use crate::model::{A, Z};
