mod z;
use foo::{B, A};
use std::io;
mod a;

fn braces() {}

mod z;
use foo::Thing as Alias;
use std::io;
mod a;

fn alias() {}

mod z;
use foo::*;
use std::io;
mod a;

fn glob() {}

mod z;
use ::foo::Thing;
use std::io;
mod a;

fn absolute() {}

mod z;
use foo::r#type;
use std::io;
mod a;

fn raw_identifier() {}

mod z;
use {std::io, crate::Thing};
use std::fmt;
mod a;

fn root_group() {}

mod z;
use super::Thing;
use std::io;
mod a;
