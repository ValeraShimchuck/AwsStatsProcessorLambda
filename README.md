# Rust AWS Lambda for stats acquiring and processing project

This is a simple Rust AWS Lambda function which can be used for stats acquiring and processing project.
It accepts events with a specific format and writes them to an S3 bucket.

## Event format

The Lambda function expects events with the following format:
```json
{
    "event": string,
    "value": number,
    "source_ip": string
}
```

## Building

To build this project for AWS Lambda:

1. Install Rust and Cargo according to official instructions: https://www.rust-lang.org/tools/install
2. Clone this repository
3. Run `cargo lambda build --release --arm64` to build the project
