use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration}; // Keep the complete run.
use other::Item;

fn line_comment_boundary() {}

use library::{RuntimeOptions, /* Keep this position. */ ApplicationConfiguration};
use other::Item;

fn internal_comment_boundary() {}

/// Keep this declaration and its neighbours.
use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration};
use other::Item;

fn documentation_boundary() {}

#[cfg(feature = "optional")]
use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration};
use other::Item;

fn attribute_boundary() {}

use library::{};
use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration};

fn unsupported_boundary() {}

#[macro_use]
mod opaque {
    use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration};
}

use application::{
    ApplicationConfiguration, ConnectionSettings, EnvironmentOverrides, RuntimeOptions,
};

fn body( ) { use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration}; let text = "use library::{Beta, Alpha};"; }
macro_rules! imports { () => { use library::{RuntimeOptions, EnvironmentOverrides, ConnectionSettings, ApplicationConfiguration}; }; }
struct Untouched { value : u8 }
