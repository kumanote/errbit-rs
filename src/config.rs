#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub host: String,
    pub project_id: String,
    pub project_key: String,
    pub environment: Option<String>,

    pub app_os: Option<String>,
    pub app_hostname: Option<String>,
    pub app_language: Option<String>,
    pub app_version: Option<String>,
    pub app_root_directory: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let host = std::env::var("AIRBRAKE_HOST").unwrap_or("https://api.airbrake.io".to_owned());
        let project_id = std::env::var("AIRBRAKE_PROJECT_ID").unwrap_or("0".to_owned());
        let project_key = std::env::var("AIRBRAKE_API_KEY").unwrap_or("0".to_owned());
        let environment = std::env::var("AIRBRAKE_ENVIRONMENT")
            .map(|inner| Some(inner))
            .unwrap_or(None);
        let app_os = Some(std::env::consts::OS.to_owned());
        let app_hostname = hostname::get()
            .map(|os_str| Some(format!("{}", os_str.to_string_lossy())))
            .unwrap_or(None);
        let app_root_directory = std::env::current_dir()
            .map(|path| Some(format!("{}", path.display())))
            .unwrap_or(None);
        Self {
            host,
            project_id,
            project_key,
            environment,
            app_os,
            app_hostname,
            app_language: None,
            app_version: None,
            app_root_directory,
        }
    }
}

impl Config {
    pub fn endpoint(&self) -> String {
        format!(
            "{}/api/v3/projects/{}/notices?key={}",
            self.host, self.project_id, self.project_key,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    #[serial_test::serial]
    fn test_default_config() {
        std::env::remove_var("AIRBRAKE_HOST");
        std::env::remove_var("AIRBRAKE_PROJECT_ID");
        std::env::remove_var("AIRBRAKE_API_KEY");
        std::env::remove_var("AIRBRAKE_ENVIRONMENT");
        let app_os = Some(std::env::consts::OS.to_owned());
        let app_hostname = Some(format!("{}", hostname::get().unwrap().to_string_lossy()));
        let app_root_directory = Some(format!("{}", std::env::current_dir().unwrap().display()));
        let config = Config::default();
        let expected = Config {
            host: "https://api.airbrake.io".to_owned(),
            project_id: "0".to_owned(),
            project_key: "0".to_owned(),
            environment: None,
            app_os,
            app_hostname,
            app_language: None,
            app_version: None,
            app_root_directory,
        };
        assert_eq!(expected, config);
        assert_eq!(
            "https://api.airbrake.io/api/v3/projects/0/notices?key=0",
            config.endpoint()
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_config() {
        std::env::set_var("AIRBRAKE_HOST", "https://errbit.example.com");
        std::env::set_var("AIRBRAKE_PROJECT_ID", "1");
        std::env::set_var("AIRBRAKE_API_KEY", "my-key");
        std::env::set_var("AIRBRAKE_ENVIRONMENT", "dev");
        let app_os = Some(std::env::consts::OS.to_owned());
        let app_hostname = Some(format!("{}", hostname::get().unwrap().to_string_lossy()));
        let app_root_directory = Some(format!("{}", std::env::current_dir().unwrap().display()));
        let config = Config::default();
        let expected = Config {
            host: "https://errbit.example.com".to_owned(),
            project_id: "1".to_owned(),
            project_key: "my-key".to_owned(),
            environment: Some("dev".to_owned()),
            app_os,
            app_hostname,
            app_language: None,
            app_version: None,
            app_root_directory,
        };
        assert_eq!(expected, config);
        assert_eq!(
            "https://errbit.example.com/api/v3/projects/1/notices?key=my-key",
            config.endpoint()
        );
    }
}
