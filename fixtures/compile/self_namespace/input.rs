mod model;

mod ordinary {
    use crate::model::branch::child;
    use crate::model::branch as Both;

    pub fn value() -> u32 {
        Both() + Both::child() + child()
    }
}

mod type_only {
    use crate::model::branch::self as selected;

    fn selected() -> u32 { 8 }

    pub fn value() -> u32 {
        selected() + selected::child()
    }
}

mod merged_type_only {
    use crate::model::branch::self as selected;
    use crate::model::branch::child;

    fn selected() -> u32 { 16 }

    pub fn value() -> u32 {
        selected() + selected::child() + child()
    }
}

mod multiple_self {
    use crate::model::branch::{self as Second};
    use crate::model::branch::{self as First, child};

    pub fn value() -> u32 {
        First::child() + Second::child() + child()
    }
}

mod qualifier_roots {
    use crate as Root;
    use crate::model::branch::child;
    use self as Here;
    use self::nested::local;
    use super as Parent;
    use super::model::branch;

    mod nested {
        pub fn local() -> u32 { 32 }
    }

    pub fn value() -> u32 {
        Root::model::branch::child() + child() + Here::nested::local() + local()
            + Parent::model::branch() + branch()
    }
}

fn main() {
    let total = ordinary::value() + type_only::value() + merged_type_only::value()
        + multiple_self::value() + qualifier_roots::value();
    println!("{total}");
}
