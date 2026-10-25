mod model;

use crate::model::{
    configuration::{load_application_configuration, resolve_environment_configuration},
    extras::*,
    metrics::{self as Metrics, collect_application_metrics},
    r#type as ApplicationType,
};

mod nested {
    pub(super) use crate::model::configuration::{
        load_application_configuration as ApplicationConfiguration,
        resolve_environment_configuration as EnvironmentConfiguration,
    };
}

fn main( ) {
    let total = load_application_configuration() + resolve_environment_configuration()
        + collect_application_metrics() + Metrics::metric_value() + ApplicationType(16).0
        + extra_value() + nested::ApplicationConfiguration() + nested::EnvironmentConfiguration();
    println!("{total}");
}
