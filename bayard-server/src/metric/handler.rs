use iron::{status, IronResult, Request, Response};
use prometheus::{Encoder, TextEncoder};

pub fn metrics(_req: &mut Request) -> IronResult<Response> {
    let metric_families = prometheus::gather();
    let mut buffer = Vec::<u8>::new();
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    let metrics_text = String::from_utf8(buffer.clone()).unwrap();

    Ok(Response::with((
        encoder.format_type(),
        status::Ok,
        metrics_text,
    )))
}
