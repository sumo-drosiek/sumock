use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::HeaderValue;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use std::sync::Mutex;

fn get_now() -> u64 {
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH)
      .expect("Time went backwards");
  return since_the_epoch.as_secs() as u64;
}

async fn handle(
  req: Request<Body>,
  metrics: Arc<Mutex<u64>>,
  logs: Arc<Mutex<u64>>,
  logs_bytes: Arc<Mutex<u64>>,
  p_metrics: Arc<Mutex<u64>>,
  p_logs: Arc<Mutex<u64>>,
  p_logs_bytes: Arc<Mutex<u64>>,
  ts: Arc<Mutex<u64>>
) -> Result<Response<Body>, Infallible> {
    let uri = req.uri().path();

    if uri.starts_with("/terraform") {
        Ok(Response::new("{\"source\": {\"url\": \"http://sumock.sumock:3000/receiver\"}}".into()))
    }
    else if uri.starts_with("/metrics-json") {
      let metrics = metrics.lock().unwrap();
      let logs = logs.lock().unwrap();
      let logs_bytes = logs_bytes.lock().unwrap();
      let p_metrics = p_metrics.lock().unwrap();
      let p_logs = p_logs.lock().unwrap();
      let p_logs_bytes = p_logs_bytes.lock().unwrap();

      Ok(Response::new(format!("{{
        \"timestamp\": {},
        \"last_minute_stats\": {{
          \"metrics\": {},
          \"logs\": {},
          \"logs_bytes\": {}
        }},
        \"total_stats\": {{
          \"metrics\": {},
          \"logs\": {},
          \"logs_bytes\": {}
        }}}}", get_now(), *p_metrics, *p_logs, *p_logs_bytes, *metrics, *logs, *logs_bytes).into()))
    }
    else if uri.starts_with("/metrics") {
      let metrics = metrics.lock().unwrap();
      let logs = logs.lock().unwrap();
      let logs_bytes = logs_bytes.lock().unwrap();

      Ok(Response::new(format!("# TYPE sumock_metrics_count counter
sumock_metrics_count {}
# TYPE sumock_logs_count counter
sumock_logs_count {}
# TYPE sumock_logs_bytes_count counter
sumock_logs_bytes_count {}", *metrics, *logs, *logs_bytes).into()))
    }
    else {
      let empty_header = HeaderValue::from_str("").unwrap();
      let content_type = req.headers().get("content-type").unwrap_or(&empty_header).to_str().unwrap();
      match content_type {
        "application/vnd.sumologic.prometheus" => {
          let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
          let vector_body = whole_body.into_iter().collect::<Vec<u8>>();
          let string_body = String::from_utf8(vector_body).unwrap();
          
          let mut metrics = metrics.lock().unwrap();
          *metrics += string_body.trim().split("\n").count() as u64;
        },
        "application/x-www-form-urlencoded" => {
          let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
          let vector_body = whole_body.into_iter().collect::<Vec<u8>>();

          let mut logs_bytes = logs_bytes.lock().unwrap();
          *logs_bytes += vector_body.len() as u64;

          let string_body = String::from_utf8(vector_body).unwrap();

          let mut logs = logs.lock().unwrap();
          *logs += string_body.trim().split("\n").count() as u64;
        },
        &_ => {
          println!("invalid header value");
        }
      }
      stats(metrics, logs, logs_bytes, p_metrics, p_logs, p_logs_bytes, ts);
      Ok(Response::new("".into()))
    }
}

async fn run_app(
  metrics: Arc<Mutex<u64>>,
  logs: Arc<Mutex<u64>>,
  logs_bytes: Arc<Mutex<u64>>,
  p_metrics: Arc<Mutex<u64>>,
  p_logs: Arc<Mutex<u64>>,
  p_logs_bytes: Arc<Mutex<u64>>,
  ts: Arc<Mutex<u64>>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Sumock is waiting for enemy on 0.0.0.0:3000!");
    let make_svc = make_service_fn(|_conn| {
      let metrics = metrics.clone();
      let logs = logs.clone();
      let logs_bytes = logs_bytes.clone();
      let p_metrics = p_metrics.clone();
      let p_logs = p_logs.clone();
      let p_logs_bytes = p_logs_bytes.clone();
      let ts = ts.clone();
      async move {
        let metrics = metrics.clone();
        let logs = logs.clone();
        let logs_bytes = logs_bytes.clone();
        let p_metrics = p_metrics.clone();
        let p_logs = p_logs.clone();
        let p_logs_bytes = p_logs_bytes.clone();
        let ts = ts.clone();
        let result = service_fn(move |req| handle(
          req,
          metrics.clone(),
          logs.clone(),
          logs_bytes.clone(),
          p_metrics.clone(),
          p_logs.clone(),
          p_logs_bytes.clone(),
          ts.clone()));
        Ok::<_, Infallible>(result)
    }});

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

}

fn stats(metrics: Arc<Mutex<u64>>, logs: Arc<Mutex<u64>>, logs_bytes: Arc<Mutex<u64>>, 
  p_metrics: Arc<Mutex<u64>>, p_logs: Arc<Mutex<u64>>, p_logs_bytes: Arc<Mutex<u64>>, ts: Arc<Mutex<u64>>) {
  let metrics = metrics.lock().unwrap();
  let logs = logs.lock().unwrap();
  let logs_bytes = logs_bytes.lock().unwrap();
  let mut p_metrics = p_metrics.lock().unwrap();
  let mut p_logs = p_logs.lock().unwrap();
  let mut p_logs_bytes = p_logs_bytes.lock().unwrap();
  let mut ts = ts.lock().unwrap();

  if get_now() >= *ts + 60 {
      println!("{} Metrics: {:10.} Logs: {:10.}; {:6.6} MB/s",
        *ts,
        *metrics - *p_metrics,
        *logs - *p_logs,
        ((*logs_bytes - *p_logs_bytes) as f64)/((get_now()-*ts) as f64)/(1e6 as f64));
      *ts = get_now();
      *p_metrics = *metrics;
      *p_logs = *logs;
      *p_logs_bytes = *logs_bytes;
  }
}

#[tokio::main]
pub async fn main() {
    let metrics = Arc::new(Mutex::new(0 as u64));
    let logs = Arc::new(Mutex::new(0 as u64));
    let logs_bytes = Arc::new(Mutex::new(0 as u64));
    let p_metrics = Arc::new(Mutex::new(0 as u64));
    let p_logs = Arc::new(Mutex::new(0 as u64));
    let p_logs_bytes = Arc::new(Mutex::new(0 as u64));
    let ts = Arc::new(Mutex::new(get_now()));

    run_app(metrics, logs, logs_bytes, p_metrics, p_logs, p_logs_bytes, ts).await;
}