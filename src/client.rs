use crate::{Notice, NotifyResult, Result};
use http::uri::InvalidUri;
use hyper::Uri;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Client {
    inner: sealed::HttpClient,
}

impl Client {
    pub fn new<U>(url: U) -> Result<Self>
    where
        U: TryInto<Uri, Error = InvalidUri>,
    {
        let url = url.try_into()?;
        let scheme = url.scheme().map(|scheme| scheme.as_str()).unwrap_or("http");
        Ok(Self {
            inner: if scheme == "https" {
                sealed::HttpClient::new_https(url)
            } else {
                sealed::HttpClient::new_http(url)
            },
        })
    }

    pub async fn notify(&self, notice: &Notice) -> Result<NotifyResult> {
        self.inner.notify(notice).await
    }
}

mod sealed {
    use crate::{Error, Notice, NotifyResult, Result};
    use http::StatusCode;
    use hyper::body::Buf;
    use hyper::client::connect::Connect;
    use hyper::client::HttpConnector;
    use hyper::{header, Uri};
    use hyper_rustls::HttpsConnector;
    use std::io::Read;

    #[derive(Debug, Clone)]
    pub struct HyperClient<C> {
        uri: Uri,
        inner: hyper::Client<C>,
    }

    impl<C> HyperClient<C> {
        pub fn new(uri: Uri, inner: hyper::Client<C>) -> Self {
            Self { uri, inner }
        }
    }

    impl<C> HyperClient<C>
    where
        C: Connect + Clone + Send + Sync + 'static,
    {
        pub async fn notify(&self, notice: &Notice) -> Result<NotifyResult> {
            let request_body = notice.to_json();
            let mut request = hyper::Request::builder()
                .method("POST")
                .uri(&self.uri)
                .body(hyper::Body::from(request_body.into_bytes()))?;
            let headers = request.headers_mut();
            headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
            let response = self.inner.request(request).await?;
            let response_status = response.status().clone();
            let mut response_body = String::new();
            hyper::body::aggregate(response.into_body())
                .await?
                .reader()
                .read_to_string(&mut response_body)?;
            if response_status == StatusCode::CREATED {
                Ok(serde_json::from_str(&response_body)?)
            } else {
                Err(Error::Gateway {
                    status_code: response_status.as_u16(),
                    reason: response_body,
                }
                .into())
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum HttpClient {
        Http(HyperClient<HttpConnector>),
        Https(HyperClient<HttpsConnector<HttpConnector>>),
    }

    impl HttpClient {
        pub fn new_http(uri: Uri) -> Self {
            Self::Http(HyperClient::new(uri, hyper::Client::new()))
        }
        pub fn new_https(uri: Uri) -> Self {
            Self::Https(HyperClient::new(
                uri,
                hyper::Client::builder().build(HttpsConnector::with_native_roots()),
            ))
        }
        pub async fn notify(&self, notice: &Notice) -> Result<NotifyResult> {
            match self {
                HttpClient::Http(c) => c.notify(notice).await,
                HttpClient::Https(c) => c.notify(notice).await,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use crate::{Config, Notice, Result};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_notify_error() -> Result<()> {
        dotenv::dotenv().ok();
        let config = Config::default();
        let client = Client::new(config.endpoint().as_str())?;
        let double_number =
            |number_str: &str| -> std::result::Result<i32, std::num::ParseIntError> {
                number_str.parse::<i32>().map(|n| 2 * n)
            };
        let err = double_number("NOT A NUMBER").err().unwrap();
        let notice = Notice::new_from_std_error(&err, &config);
        let result = client.notify(&notice).await?;
        assert!(!result.id.is_empty());
        Ok(())
    }
}
