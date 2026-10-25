mod z;
use foo::{B, a::{self, A}};
mod a;

fn self_element() {}

use foo::{Z, a::{B as Alias, A}};
use bar::{B, A};

fn alias() {}

use foo::{Z, a::{Trait as _, A}};
use bar::{B, A};

fn anonymous_alias() {}

use foo::{Z, a::{*, A}};
use bar::{B, A};

fn glob() {}

use ::foo::{B, A};
use bar::{B, A};

fn absolute_path() {}

use foo::{Z, a::{r#type, A}};
use bar::{B, A};

fn raw_identifier() {}

use foo::{Z, self::a::A};
use bar::{B, A};

fn nested_qualifier() {}

use foo::{Z, A};
mod r#type;

fn raw_module() {}

use foo::{Z, a::{B, A}};
