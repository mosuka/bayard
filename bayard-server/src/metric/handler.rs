use hyper::{Body, Method, Request, Response, StatusCode};
use prometheus::{Encoder, TextEncoder};

pub async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let metric_families = prometheus::gather();
            let mut buffer = Vec::<u8>::new();
            let encoder = TextEncoder::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let metrics_text = String::from_utf8(buffer.clone()).unwrap();

            *response.status_mut() = StatusCode::OK;
            *response.body_mut() = Body::from(metrics_text);
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}
