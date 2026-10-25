mod outer {
    use library::{ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides};

    mod middle {
        mod inner {
            use library::{
                ApplicationConfiguration, ConnectionPoolConfiguration, EnvironmentOverrides,
            };
        }
    }
}
