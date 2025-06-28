// src/lib/logging/elk.rs
use serde_json::{json, Value};
use tracing::{Subscriber, subscriber::set_global_default};
use tracing_subscriber::{
    fmt::{format::Writer, time::FormatTime, MakeWriter},
    layer::SubscriberExt,
    registry::LookupSpan,
    Layer,
};
use async_channel::{bounded, Receiver, Sender};
use reqwest::Client;
use std::{io, time::SystemTime};
use thiserror::Error;
use tokio::task::JoinHandle;

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("Logging initialization failed")]
    InitializationError,
    #[error("Log transport error: {0}")]
    TransportError(#[from] reqwest::Error),
}

pub struct ElkLogger {
    sender: Sender<Value>,
    handle: JoinHandle<()>,
}

impl ElkLogger {
    pub fn new(
        logstash_url: &str,
        service_name: &str,
        environment: &str,
    ) -> Result<Self, LoggingError> {
        let (sender, receiver) = bounded(1000);
        let client = Client::new();
        let url = logstash_url.to_string();
        let service = service_name.to_string();
        let env = environment.to_string();

        let handle = tokio::spawn(async move {
            Self::log_consumer(receiver, client, url, service, env).await;
        });

        Ok(Self { sender, handle })
    }

    pub fn init_logging(&self) -> Result<(), LoggingError> {
        let elk_layer = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_writer(ElkLogWriter {
                sender: self.sender.clone(),
            })
            .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

        let subscriber = tracing_subscriber::registry()
            .with(elk_layer)
            .with(tracing_subscriber::fmt::layer().pretty());

        set_global_default(subscriber).map_err(|_| LoggingError::InitializationError)?;
        Ok(())
    }

    async fn log_consumer(
        receiver: Receiver<Value>,
        client: Client,
        url: String,
        service: String,
        env: String,
    ) {
        let mut batch = Vec::with_capacity(100);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    if let Ok(msg) = msg {
                        let enhanced = Self::enhance_log_entry(msg, &service, &env);
                        batch.push(enhanced);
                        
                        if batch.len() >= 100 {
                            if let Err(e) = Self::send_batch(&client, &url, &batch).await {
                                eprintln!("Error sending logs: {}", e);
                            }
                            batch.clear();
                        }
                    }
                }
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        if let Err(e) = Self::send_batch(&client, &url, &batch).await {
                            eprintln!("Error sending logs: {}", e);
                        }
                        batch.clear();
                    }
                }
            }
        }
    }

    fn enhance_log_entry(mut entry: Value, service: &str, env: &str) -> Value {
        let map = entry.as_object_mut().unwrap();
        
        map.insert("@timestamp".to_string(), json!(SystemTime::now()));
        map.insert("service.name".to_string(), json!(service));
        map.insert("environment".to_string(), json!(env));
        
        if let Some(fields) = map.get_mut("fields") {
            if let Some(request_id) = fields.get("request_id") {
                map.insert("trace.id".to_string(), request_id.clone());
            }
            
            if let Some(duration) = fields.get("duration_ms") {
                map.insert("event.duration".to_string(), json!(duration.as_f64().unwrap() * 1_000_000.0));
            }
        }
        
        entry
    }

    async fn send_batch(client: &Client, url: &str, batch: &[Value]) -> Result<(), reqwest::Error> {
        client
            .post(url)
            .json(batch)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

struct ElkLogWriter {
    sender: Sender<Value>,
}

impl ElkLogWriter {
    fn format_record(&self, event: &tracing::Event<'_>) -> Value {
        let mut fields = json!({});
        let mut visitor = JsonVisitor(&mut fields);
        event.record(&mut visitor);

        json!({
            "message": fields.get("message"),
            "log.level": event.metadata().level().to_string(),
            "fields": fields,
        })
    }
}

impl MakeWriter<'_> for ElkLogWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        self.clone()
    }
}

impl io::Write for ElkLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let value = serde_json::from_slice(buf).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, e)
        })?;
        
        self.sender.send_blocking(value).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        })?;
        
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct JsonVisitor<'a>(&'a mut Value);

impl tracing::field::Visit for JsonVisitor<'_> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0[field.name()] = json!(value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0[field.name()] = json!(format!("{:?}", value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0[field.name()] = json!(value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0[field.name()] = json!(value);
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0[field.name()] = json!(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[tokio::test]
    async fn test_logging() {
        let mock_server = MockServer::start();
        let logger = ElkLogger::new(
            &mock_server.uri(),
            "test-service",
            "testing"
        ).unwrap();
        
        logger.init_logging().unwrap();

        info!(request_id = "1234", duration_ms = 42, "Test message");
        
        // Verify mock server received logs
    }
}