# errbit client

> Rust implementation of errbit client that is compatible with airbrake client.

This package provides a simple utility to report errors to your [errbit](https://github.com/errbit/errbit) server.

**Notes**

- This crate supports [anyhow](https://github.com/dtolnay/anyhow) Error type for reporting with backtrace.
- There will be no backtrace output for `std::error::Error`.

## Installation

#### Dependencies

- [Rust with Cargo](http://rust-lang.org)

**rust-toolchain**

```text
1.54.0
```

#### Importing

**~/.cargo/config**

```toml
[net]
git-fetch-with-cli = true
```

**Cargo.toml**

```toml
[dependencies]
errbit = { version = "0.1.0", git = "ssh://git@github.com/kumanote/errbit-rs.git", branch = "main" }
```

## Configurations

You can set your default `host`/`project id`/`project key`/`environment` values by setting the following environment
variables.

| ENV VAR NAME | CONTENT | EXAMPLE |
| --- | --- | --- |
| `AIRBRAKE_HOST` | your errbit server host name | `https://api.airbrake.io` |
| `AIRBRAKE_PROJECT_ID` | your errbit project id | `1` |
| `AIRBRAKE_PROJECT_ID` | your errbit project api key | `ffcbf68d38782ae9ba32591a859f1452` |
| `AIRBRAKE_ENVIRONMENT` | your application environment | `development` / `dev` / `staging` |



## Examples

Here's a basic example:

```rust
use errbit::{Config, Notice, Notifier, Result};

#[tokio::main]
async fn main() -> Result<()>  {
    let mut config = Config::default();
    config.host = "https://errbit.yourdomain.com".to_owned();
    config.project_id = "1".to_owned();
    config.project_key = "ffffffffffffffffffffffffffffffff".to_owned();
    config.environment = Some("staging".to_owned());
    let notifier = Notifier::new(config)?;
    let double_number =
        |number_str: &str| -> std::result::Result<i32, std::num::ParseIntError> {
            number_str.parse::<i32>().map(|n| 2 * n)
        };
    let err = double_number("NOT A NUMBER").err().unwrap();
    let result = notifier.notify_error(err).await?;
    println!("{}", result.id);
    Ok(())
}
```
