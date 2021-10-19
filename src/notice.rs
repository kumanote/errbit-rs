use crate::Config;
use serde::{Deserialize, Serialize, Serializer};
use serde_json;
use std::collections::HashMap;
use std::fmt;

/// @see https://airbrake.io/docs/api/#create-notice-v3
#[derive(Debug, Serialize)]
pub struct Notice {
    pub errors: Vec<ErrorInfo>,
    pub context: Context,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

impl Notice {
    pub fn new_from_std_error<E: std::error::Error>(error: &E, config: &Config) -> Self {
        let error_info = ErrorInfo::new_with_error(error);
        let mut context = Context::new_from_config(&config);
        context.severity = Some(Severity::ERROR);
        Self {
            errors: vec![error_info],
            context,
            environment: None,
            session: None,
            params: None,
        }
    }
    pub fn new_from_anyhow_error(error: &anyhow::Error, config: &Config) -> Self {
        let error_info = ErrorInfo::from(error);
        let mut context = Context::new_from_config(&config);
        context.severity = Some(Severity::ERROR);
        Self {
            errors: vec![error_info],
            context,
            environment: None,
            session: None,
            params: None,
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<Vec<BacktraceInfo>>,
}

impl ErrorInfo {
    pub fn new_with_error<E: std::error::Error>(error: &E) -> Self {
        let type_ = format!("{:?}", error)
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned();
        let message = format!("{}", error);
        Self {
            type_,
            message,
            backtrace: None,
        }
    }
}

impl From<&anyhow::Error> for ErrorInfo {
    fn from(error: &anyhow::Error) -> Self {
        let type_ = format!("{:?}", error.root_cause())
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned();
        let message = format!("{}", error);
        let backtrace_string = format!("{}", error.backtrace());
        let backtraces: Vec<&str> = backtrace_string
            .split("\n")
            .into_iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim())
            .collect();
        let mut backtrace_infos = vec![];
        let mut item = BacktraceInfo::default();
        let mut backtrace_iter = backtraces.into_iter();
        loop {
            if let Some(t) = backtrace_iter.next() {
                if t.starts_with("at ") {
                    let position_part = &t[3..];
                    let position_info: Vec<&str> = position_part.split(":").into_iter().collect();
                    if position_info.len() > 0 {
                        item.file = Some(position_info[0].to_owned())
                    }
                    if position_info.len() > 1 {
                        if let Ok(l) = position_info[1].parse::<usize>() {
                            item.line = Some(l)
                        }
                    }
                    if position_info.len() > 2 {
                        if let Ok(c) = position_info[2].parse::<usize>() {
                            item.column = Some(c)
                        }
                    }
                    backtrace_infos.push(item.clone());
                    item = BacktraceInfo::default();
                } else {
                    item = BacktraceInfo::default();
                    item.function = Some(t.to_owned())
                }
            } else {
                if !item.is_empty() {
                    backtrace_infos.push(item.clone())
                }
                break;
            }
        }
        Self {
            type_,
            message,
            backtrace: Some(backtrace_infos),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BacktraceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<HashMap<String, String>>,
}

impl BacktraceInfo {
    pub fn is_empty(&self) -> bool {
        self.file.is_none()
            && self.function.is_none()
            && self.line.is_none()
            && self.column.is_none()
            && self.code.is_none()
    }
}

impl Default for BacktraceInfo {
    fn default() -> Self {
        Self {
            file: None,
            function: None,
            line: None,
            column: None,
            code: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifier: Option<NotifierInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_severity"
    )]
    pub severity: Option<Severity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "userAgent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(rename = "userAddr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_addr: Option<String>,
    #[serde(rename = "remoteAddr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addr: Option<String>,
    #[serde(rename = "rootDirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(rename = "httpMethod")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
}

impl Context {
    pub fn new_from_config(config: &Config) -> Self {
        Self {
            notifier: Some(NotifierInfo::default()),
            environment: config.environment.clone(),
            severity: None,
            component: None,
            action: None,
            os: config.app_os.clone(),
            hostname: config.app_hostname.clone(),
            language: config.app_language.clone(),
            version: config.app_version.clone(),
            url: None,
            user_agent: None,
            user_addr: None,
            remote_addr: None,
            root_directory: config.app_root_directory.clone(),
            user: None,
            route: None,
            http_method: None,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            notifier: None,
            environment: None,
            severity: None,
            component: None,
            action: None,
            os: None,
            hostname: None,
            language: None,
            version: None,
            url: None,
            user_agent: None,
            user_addr: None,
            remote_addr: None,
            root_directory: None,
            user: None,
            route: None,
            http_method: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

fn serialize_severity<S>(value: &Option<Severity>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let severity = if let Some(severity) = value {
        severity
    } else {
        &Severity::INVALID
    };
    serializer.serialize_str(severity.to_string().as_str())
}

#[derive(Debug, Serialize)]
pub struct NotifierInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Default for NotifierInfo {
    fn default() -> Self {
        Self {
            name: Some(format!("{}", env!("CARGO_PKG_NAME"))),
            version: Some(format!("{}", env!("CARGO_PKG_VERSION"))),
            url: Some(format!("{}", env!("CARGO_PKG_REPOSITORY"))),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Severity {
    DEBUG,
    INFO,
    NOTICE,
    WARNING,
    ERROR,
    CRITICAL,
    ALERT,
    EMERGENCY,
    INVALID,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::DEBUG => write!(f, "debug"),
            Severity::INFO => write!(f, "info"),
            Severity::NOTICE => write!(f, "notice"),
            Severity::WARNING => write!(f, "warning"),
            Severity::ERROR => write!(f, "error"),
            Severity::CRITICAL => write!(f, "critical"),
            Severity::ALERT => write!(f, "alert"),
            Severity::EMERGENCY => write!(f, "emergency"),
            Severity::INVALID => write!(f, "invalid"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NotifyResult {
    pub id: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::{Context as ErrorContext, ErrorInfo, Notice, NotifierInfo, Severity};
    use anyhow::{Context, Result};

    #[test]
    fn test_error_info_from_std_error() {
        let double_number =
            |number_str: &str| -> std::result::Result<i32, std::num::ParseIntError> {
                number_str.parse::<i32>().map(|n| 2 * n)
            };
        let err = double_number("NOT A NUMBER").err().unwrap();
        let error_info = ErrorInfo::new_with_error(&err);
        assert!(error_info.backtrace.is_none());
    }

    #[test]
    fn test_error_info_from_anyhow() {
        let double_number = |number_str: &str| -> Result<i32> {
            number_str
                .parse::<i32>()
                .map(|n| 2 * n)
                .with_context(|| format!("Failed to parse number_str of {}", number_str))
        };
        let err = double_number("NOT A NUMBER").err().unwrap();
        let error_info = ErrorInfo::from(&err);
        assert!(error_info.backtrace.unwrap().len() > 0);
    }

    #[test]
    fn test_notice() {
        let error_info = ErrorInfo {
            type_: "Error".to_owned(),
            message: "This is test".to_owned(),
            backtrace: None,
        };
        let mut context = ErrorContext::default();
        context.http_method = Some("GET".to_owned());
        let notice = Notice {
            errors: vec![error_info],
            context,
            environment: None,
            session: None,
            params: Some(
                [("param1".to_owned(), "1".to_owned())]
                    .iter()
                    .cloned()
                    .collect(),
            ),
        };
        let json = notice.to_json();
        let expected = r##"{"errors":[{"type":"Error","message":"This is test"}],"context":{"httpMethod":"GET"},"params":{"param1":"1"}}"##;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_notice2() {
        let error_info = ErrorInfo {
            type_: "Error".to_owned(),
            message: "This is test".to_owned(),
            backtrace: None,
        };
        let mut context = ErrorContext::default();
        context.http_method = Some("POST".to_owned());
        context.severity = Some(Severity::INFO);
        context.notifier = Some(NotifierInfo::default());
        let notice = Notice {
            errors: vec![error_info],
            context,
            environment: None,
            session: None,
            params: None,
        };
        let json = notice.to_json();
        let expected = r##"{"errors":[{"type":"Error","message":"This is test"}],"context":{"notifier":{"name":"errbit","version":"0.1.0","url":"https://github.com/kumanote/errbit-rs"},"severity":"info","httpMethod":"POST"}}"##;
        assert_eq!(json, expected);
    }
}
