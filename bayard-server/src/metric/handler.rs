use actix_web::{get, Error, HttpResponse};
use prometheus::{Encoder, TextEncoder};

#[get("/metrics")]
pub async fn metrics() -> Result<HttpResponse, Error> {
    let metric_families = prometheus::gather();
    let mut buffer = Vec::<u8>::new();
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    let metrics_text = String::from_utf8(buffer.clone()).unwrap();

    let res = HttpResponse::Ok().body(metrics_text);
    Ok(res)
}
