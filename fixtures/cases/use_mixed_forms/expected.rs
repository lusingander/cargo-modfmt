mod a;
mod z;

use std::io;

use foo::{A, B};

fn braces() {}

mod a;
mod z;

use std::io;

use foo::Thing as Alias;

fn alias() {}

mod a;
mod z;

use std::io;

use foo::*;

fn glob() {}

mod a;
mod z;

use std::io;

use ::foo::Thing;

fn absolute() {}

mod a;
mod z;

use std::io;

use foo::r#type;

fn raw_identifier() {}

mod z;
use {std::io, crate::Thing};
use std::fmt;
mod a;

fn root_group() {}

mod a;
mod z;

use std::io;

use super::Thing;
