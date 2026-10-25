mod engine;

pub mod model;

use std::{collections::BTreeMap, path::Path};

use engine::run;

use crate::model::{
    configuration::{load_application_configuration, resolve_environment_configuration},
    extras::*,
    metrics::{self as Metrics, collect_application_metrics},
    r#type as ApplicationType,
};

#[cfg(all())]
mod worker {
    pub(super) use crate::model::configuration::{
        load_application_configuration as Load, resolve_environment_configuration as Resolve,
    };

    pub fn value() -> usize { Load() + Resolve() }
}

mod commented {
    // Keep the imports with this explanation.
    use std::path::Path;
    use std::collections::BTreeMap;

    pub fn value() -> usize {
        Path::new("kept").as_os_str().len() + BTreeMap::<u8, u8>::new().len()
    }
}

mod attributed {
    #[allow(unused_imports)]
    use std::path::Path;
    use std::collections::BTreeMap;

    pub fn value() -> usize {
        Path::new("attr").as_os_str().len() + BTreeMap::<u8, u8>::new().len()
    }
}

mod unsupported {
    use {std::path::Path, std::collections::BTreeMap};

    pub fn value() -> usize {
        Path::new("tree").as_os_str().len() + BTreeMap::<u8, u8>::new().len()
    }
}

#[cfg_attr(all(), allow(unused_imports))]
mod opaque {
    use std::path::Path;
    use std::collections::BTreeMap;

    pub fn value() -> usize {
        Path::new("opaque").as_os_str().len() + BTreeMap::<u8, u8>::new().len()
    }
}

macro_rules! local_value {
    () => {{
        use std::path::Path;
        use std::collections::BTreeMap;
        Path::new("macro").as_os_str().len() + BTreeMap::<u8, u8>::new().len()
    }};
}

use crate::model::{
    configuration::load_application_configuration as LoadAgain,
    metrics::metric_value as MetricAgain,
};

fn main( ) {
    use std::path::Path as LocalPath;
    use std::collections::BTreeMap as LocalMap;
    let source = "use z::Z; use a::A;";
    let total = run(load_application_configuration() + resolve_environment_configuration())
        + collect_application_metrics() + Metrics::metric_value() + ApplicationType(17).0
        + extra_value() + worker::value() + commented::value() + attributed::value()
        + unsupported::value() + opaque::value() + local_value!() + MetricAgain() + LoadAgain()
        + Path::new("root").as_os_str().len() + LocalPath::new("body").as_os_str().len()
        + BTreeMap::<u8, u8>::new().len() + LocalMap::<u8, u8>::new().len();
    assert_eq!(source, "use z::Z; use a::A;");
    println!("{total}");
}
