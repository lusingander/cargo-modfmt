use foo::Item as Zed;
use foo::Trait as _;
use foo::Item;
use foo::Item as Alpha;
use foo::Other as _;

fn distinct_aliases() {}

use foo::Item as Same;
use foo::Other as Same;
