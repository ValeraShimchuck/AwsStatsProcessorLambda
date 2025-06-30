use std::time::{SystemTime, UNIX_EPOCH};

use aws_config::{load_defaults, BehaviorVersion};
use aws_sdk_s3::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Number, Value};
use tracing::{info, Level};

struct EventData<'a> {
    event: &'a Value,
    value: &'a Number,
    source_ip: &'a Value
}

fn get_event<'a>(data: &'a Value) -> Option<EventData<'a>> {
    Some(EventData {
        event: data.get("event")?,
        value: data.get("value")?.as_number()?,
        source_ip: data.get("source_ip")?,
    })
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {

    let (event, _context) = event.into_parts();
    let event_data = match get_event(&event) {
        Some(data) => data,
        None => {
            info!("Rejected invalid data: {}", event);
            return Ok(json!({"status": "rejected"}));
        }
    };

    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_millis();

    let s3_data = json!({
        "timestamp": timestamp,
        "event": event_data.event,
        "value": event_data.value,
        "source_ip": event_data.source_ip,
    });

    let config = load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    client
        .put_object()
        .bucket(std::env::var("BUCKET_NAME")?)
        .key(format!("data/{}.json", uuid::Uuid::new_v4()))
        .body(s3_data.to_string().as_bytes().to_owned().into())
        .content_type("application/json")
        .send()
        .await?;
    Ok(json!({"status": "accepted"}))
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    lambda_runtime::run(service_fn(handler)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let valid = json!({"event": "login", "value": 1.0, "source_ip": "192.168.0.1"});
        assert!(get_event(&valid).is_some());

        let invalid = json!({"event": "login", "value": "not_number"});
        assert!(get_event(&invalid).is_none());
    }

    #[tokio::test]
    async fn test_handler() {
        let event = LambdaEvent::new(
            json!({"event": "login", "value": 1.0, "source_ip": "192.168.0.1"}), 
            lambda_runtime::Context::default()
        );
        
        let result = handler(event).await.unwrap();
        assert_eq!(result, json!({"status": "accepted"}));
    }
}