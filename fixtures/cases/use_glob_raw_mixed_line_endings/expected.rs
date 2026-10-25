use ::foo::r#type;
use ::foo::*;

fn between_runs() {}

use foo::B;
use foo::{
    r#type,
    *,
};

fn after_nested_mixed() {}

use ::foo::{Z, r#type, *};
