mod client;
mod config;
mod error;
mod notice;
mod notifier;

pub use client::Client;
pub use config::Config;
pub use error::{Error, Result};
pub use notice::*;
pub use notifier::Notifier;

#[cfg(test)]
mod tests {
    use crate::{Config, Notice, Notifier, Result};
    use anyhow::Context;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_notify_error() -> Result<()> {
        dotenv::dotenv().ok();
        let config = Config::default();
        let notifier = Notifier::new(config)?;
        let double_number =
            |number_str: &str| -> std::result::Result<i32, std::num::ParseIntError> {
                number_str.parse::<i32>().map(|n| 2 * n)
            };
        let err = double_number("NOT A NUMBER").err().unwrap();
        let result = notifier.notify_error(&err).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.id.is_empty());
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_notify_anyhow_error() -> Result<()> {
        dotenv::dotenv().ok();
        let config = Config::default();
        let notifier = Notifier::new(config)?;
        let double_number = |number_str: &str| -> Result<i32> {
            number_str
                .parse::<i32>()
                .map(|n| 2 * n)
                .with_context(|| format!("Failed to parse number_str of {}", number_str))
        };
        let err = double_number("NOT A NUMBER").err().unwrap();
        let result = notifier.notify_anyhow_error(&err).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.id.is_empty());
        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_notify() -> Result<()> {
        dotenv::dotenv().ok();
        let config = Config::default();
        let notifier = Notifier::new(config.clone())?;
        let double_number = |number_str: &str| -> Result<i32> {
            number_str
                .parse::<i32>()
                .map(|n| 2 * n)
                .with_context(|| format!("Failed to parse number_str of {}", number_str))
        };
        let err = double_number("NOT A NUMBER").err().unwrap();
        let mut notice = Notice::new_from_anyhow_error(&err, &config);
        notice.context.user_agent = Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.100 Safari/537.36".to_owned());
        let result = notifier.notify(notice).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.id.is_empty());
        Ok(())
    }
}
