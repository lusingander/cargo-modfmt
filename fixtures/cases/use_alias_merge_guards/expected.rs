use bar::{A, B as Bee};
use foo::A as Same;
use foo::A as Same;
use foo::Z as Last;

fn duplicate_named_alias() {}

use foo::Other;
use foo::Trait as _;
use foo::Trait as _;

fn duplicate_underscore() {}

use foo::{self as Same};
use foo::{self as Same, Z as Last};

fn equivalent_self_duplicate() {}

use foo::Z;
use foo::a as ModuleAlias;
use foo::a::Child as ChildAlias;

fn ordinary_leaf_and_child() {}

use foo as Root;
use foo::Z;

fn ordinary_root_and_child() {}

use foo::a as Both;
use foo::a::{self as TypeOnly, A, Z};

fn normal_and_self_at_same_path() {}

use foo::Z;
use foo::{Item as Same, Item as Same};

fn repeated_element() {}

use foo as Zed;
use foo as Alpha;

fn ordinary_root_aliases() {}

use crate as Root;
use crate::Z;
use crate::{self as Root};
