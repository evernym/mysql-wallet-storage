use std::env;

pub struct Config {
    config: String,
    credentials: String,
}

impl Config {

    pub fn new() -> Config {
        Config {
            config: json!(
                {
                    "read_host": env::var("DB_READ_HOST").unwrap_or("wallet".to_string()),
                    "write_host": env::var("DB_WRITE_HOST").unwrap_or("wallet".to_string()),
                    "port": env::var("DB_PORT").unwrap_or(3306.to_string()).parse::<u32>().unwrap(),
                    "db_name": env::var("DB_NAME").unwrap_or("wallet".to_string())
                }
            ).to_string(),
            credentials: json!(
                {
                    "user": env::var("DB_USER").unwrap_or("wallet".to_string()),
                    "pass": env::var("DB_PASS").unwrap_or("wallet".to_string())
                }
            ).to_string()
        }
    }

    pub fn get_config(&self) -> String {
        self.config.clone()
    }

    pub fn get_credentials(&self) -> String {
        self.credentials.clone()
    }

}
