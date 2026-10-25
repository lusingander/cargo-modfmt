mod compact { use library::{
    ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides,
}; }
fn boundary( ) { } use library::{
    ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides,
};
mod ordered { use alibrary::{
    ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides,
};
use zlibrary::{ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides}; }
mod contracted { use foo::{A, B}; } mod after_contract { use library::{
    ApplicationConfiguration, ConnectionSettings,
}; }
mod expanded { use library::{
    ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides,
}; } mod after_expand { use library::{ApplicationConfiguration, ConnectionSettings}; }
