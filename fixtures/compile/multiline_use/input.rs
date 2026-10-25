use crate::model::configuration::resolve_environment_configuration;
use crate::model::metrics::{self as Metrics, collect_application_metrics};
mod model;
use crate::model::configuration::load_application_configuration;
use crate::model::r#type as ApplicationType;
use crate::model::extras::*;

mod nested {
    pub(super) use crate::model::configuration::resolve_environment_configuration as EnvironmentConfiguration;
    pub(super) use crate::model::configuration::load_application_configuration as ApplicationConfiguration;
}

fn main( ) {
    let total = load_application_configuration() + resolve_environment_configuration()
        + collect_application_metrics() + Metrics::metric_value() + ApplicationType(16).0
        + extra_value() + nested::ApplicationConfiguration() + nested::EnvironmentConfiguration();
    println!("{total}");
}
