use std::collections::HashMap;

use prometheus::{CounterVec, Encoder, Opts, Registry, TextEncoder};

pub struct Metrics {
    registry: Registry,
    request_counter: CounterVec,
}

impl Metrics {
    pub fn new(id: u64) -> Metrics {
        let request_counter_opts = Opts::new("bayard_requests_total", "Total number of requests.")
            .const_label("id", &id.to_string());
        let request_counter = CounterVec::new(request_counter_opts, &["request_type"]).unwrap();

        let registry = Registry::new();
        registry
            .register(Box::new(request_counter.clone()))
            .unwrap();

        Metrics {
            registry,
            request_counter,
        }
    }

    pub fn inc_request_count(&mut self, request_type: &str) {
        let mut labels = HashMap::new();
        labels.insert("request_type", request_type);

        self.request_counter.with(&labels).inc();
    }

    pub fn get_metrics(&mut self) -> String {
        let mut buffer = Vec::<u8>::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let metrics = String::from_utf8(buffer.clone()).unwrap();

        buffer.clear();

        metrics
    }
}
