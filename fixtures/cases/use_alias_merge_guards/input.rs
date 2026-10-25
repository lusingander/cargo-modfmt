use foo::Z as Last;
use bar::B as Bee;
use foo :: A as Same;
use foo::A as Same;
use bar::A;

fn duplicate_named_alias() {}

use foo::Trait as _;
use foo::Other;
use foo::Trait as _;

fn duplicate_underscore() {}

use foo::{self as Same, Z as Last};
use foo::self as Same;

fn equivalent_self_duplicate() {}

use foo::a::Child as ChildAlias;
use foo::a as ModuleAlias;
use foo::Z;

fn ordinary_leaf_and_child() {}

use foo::Z;
use foo as Root;

fn ordinary_root_and_child() {}

use foo::a::{self as TypeOnly, Z, A};
use foo::a as Both;

fn normal_and_self_at_same_path() {}

use foo::Z;
use foo::{Item as Same, Item as Same};

fn repeated_element() {}

use foo as Zed;
use foo as Alpha;

fn ordinary_root_aliases() {}

use crate::Z;
use crate as Root;
use crate::{self as Root};
