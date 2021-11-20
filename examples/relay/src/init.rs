pub mod once {
    use std::sync::Once;

    use tracing::{subscriber::set_global_default, Subscriber};
    use tracing_log::LogTracer;
    use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

    static INIT: Once = Once::new();

    fn get_subscriber() -> impl Subscriber + Send + Sync {
        let env_filter = EnvFilter::new("trace");
        let stdout_layer = tracing_subscriber::fmt::layer().pretty();
        Registry::default().with(env_filter).with(stdout_layer)
    }

    pub fn init() {
        INIT.call_once(|| {
            let subscriber = get_subscriber();
            LogTracer::init().expect("Failed to initialize logger");
            set_global_default(subscriber).expect("Failed to set subscriber");
        });
    }
}
