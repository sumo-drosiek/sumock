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
  ts: Arc<Mutex<u64>>
) -> Result<Response<Body>, Infallible> {
    let uri = req.uri().path();

    if uri.starts_with("/terraform") {
        Ok(Response::new("{\"source\": {\"url\": \"http://sumock.sumock:3000/receiver\"}}".into()))
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
          let string_body = String::from_utf8(vector_body).unwrap();

          let mut logs = logs.lock().unwrap();
          *logs += string_body.trim().split("\n").count() as u64;
        },
        &_ => {
          println!("invalid header value");
        }
      }
      stats(metrics, logs, ts);
      Ok(Response::new("".into()))
    }
}

async fn run_app(metrics: Arc<Mutex<u64>>, logs: Arc<Mutex<u64>>, ts: Arc<Mutex<u64>>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Sumock is waiting for enemy on 0.0.0.0:3000!");
    let make_svc = make_service_fn(|_conn| {
      let metrics = metrics.clone();
      let logs = logs.clone();
      let ts = ts.clone();
      async move {
        let metrics = metrics.clone();
        let logs = logs.clone();
        let ts = ts.clone();
        let result = service_fn(move |req| handle(req, metrics.clone(), logs.clone(), ts.clone()));
        Ok::<_, Infallible>(result)
    }});

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

}

fn stats(metrics: Arc<Mutex<u64>>, logs: Arc<Mutex<u64>>, ts: Arc<Mutex<u64>>) {
  let mut metrics = metrics.lock().unwrap();
  let mut logs = logs.lock().unwrap();
  let mut ts = ts.lock().unwrap();

  if get_now() >= *ts + 60 {
      println!("{} Metrics: {:10.} Logs: {:10.}", *ts, *metrics, *logs);
      *ts = get_now();
      *metrics = 0;
      *logs = 0;
  }
}

#[tokio::main]
pub async fn main() {
    let metrics = Arc::new(Mutex::new(0 as u64));
    let logs = Arc::new(Mutex::new(0 as u64));
    let ts = Arc::new(Mutex::new(get_now()));

    run_app(metrics, logs, ts).await;
}