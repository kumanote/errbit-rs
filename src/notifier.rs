use crate::{Client, Config, Notice, NotifyResult, Result};

#[derive(Debug, Clone)]
pub struct Notifier {
    config: Config,
    client: Client,
}

impl Notifier {
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::new(config.endpoint().as_str())?;
        Ok(Self { config, client })
    }

    pub async fn notify(&self, notice: Notice) -> Result<NotifyResult> {
        self.client.notify(&notice).await
    }

    pub async fn notify_error<E: std::error::Error>(&self, error: &E) -> Result<NotifyResult> {
        let notice = Notice::new_from_std_error(error, &self.config);
        self.client.notify(&notice).await
    }

    pub async fn notify_anyhow_error(&self, error: &anyhow::Error) -> Result<NotifyResult> {
        let notice = Notice::new_from_anyhow_error(error, &self.config);
        self.client.notify(&notice).await
    }
}
