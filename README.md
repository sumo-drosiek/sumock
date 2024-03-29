# Sumock

Sumock is an small contenerised application written in rust which can be used for local testing of the [`kubernetes sumologic collection`](https://github.com/SumoLogic/sumologic-kubernetes-collection)

## Running

```
cargo run
```

Sumock is listening on port 3000. For now it cannot get any arguments

## Terraform mock

It expose the `/terraform.*` url which can be used to set HTTP source for k8s collection to sumock itself

## Statistics

There are endpoints which provides statistics:
 * `metrics` - exposes sumock metrics in prometheus format
  ```
  # TYPE sumock_metrics_count counter
  sumock_metrics_count 123
  # TYPE sumock_logs_count counter
  sumock_logs_count 123
  # TYPE sumock_logs_bytes_count counter
  sumock_logs_bytes_count 45678
  ```
 * `/metrics-list` - returns list of counted unique metrics
  ```
  prometheus_remote_storage_shards: 100
  prometheus_remote_storage_shards_desired: 100
  prometheus_remote_storage_shards_max: 100
  prometheus_remote_storage_shards_min: 100
  prometheus_remote_storage_string_interner_zero_reference_releases_total: 10
  prometheus_remote_storage_succeeded_samples_total: 100
  ```
 * `/metrics-reset` - reset the metrics counter (zeroes `/metrics-list`)

# Disclaimer

This tool is not intended to be used by the 3rd party. It can significantly change behavior and should be treated as experimental.