mod outer {
    use library::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration};

    mod middle {
        mod inner {
            use library::{EnvironmentOverrides, ConnectionPoolConfiguration, ApplicationConfiguration};
        }
    }
}
