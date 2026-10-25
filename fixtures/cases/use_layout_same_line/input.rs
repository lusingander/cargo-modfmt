mod compact { use library::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration}; }
fn boundary( ) { } use library::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration};
mod ordered { use zlibrary::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration}; use alibrary::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration}; }
mod contracted { use foo::{
    B,
    A,
}; } mod after_contract { use library::{ConnectionSettings, ApplicationConfiguration}; }
mod expanded { use library::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration}; } mod after_expand { use library::{ConnectionSettings, ApplicationConfiguration}; }
