pub mod configuration {
    pub fn load_application_configuration() -> usize {
        2
    }

    pub fn resolve_environment_configuration() -> usize {
        3
    }
}

pub mod metrics {
    pub fn collect_application_metrics() -> usize {
        5
    }

    pub fn metric_value() -> usize {
        7
    }
}

pub mod extras {
    pub fn extra_value() -> usize {
        11
    }
}

pub struct ApplicationType(pub usize);

pub use self::ApplicationType as r#type;
