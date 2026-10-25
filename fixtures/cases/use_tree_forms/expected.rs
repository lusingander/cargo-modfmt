mod a;
mod z;

use foo::{
    B,
    a::{self, A},
};

fn self_element() {}

use bar::{A, B};
use foo::{
    Z,
    a::{A, B as Alias},
};

fn alias() {}

use bar::{A, B};
use foo::{
    Z,
    a::{A, Trait as _},
};

fn anonymous_alias() {}

use bar::{A, B};
use foo::{
    Z,
    a::{A, *},
};

fn glob() {}

use ::foo::{A, B};
use bar::{A, B};

fn absolute_path() {}

use bar::{A, B};
use foo::{
    Z,
    a::{A, r#type},
};

fn raw_identifier() {}

use foo::{Z, self::a::A};
use bar::{B, A};

fn nested_qualifier() {}

mod r#type;

use foo::{A, Z};

fn raw_module() {}

use foo::{
    Z,
    a::{A, B},
};
