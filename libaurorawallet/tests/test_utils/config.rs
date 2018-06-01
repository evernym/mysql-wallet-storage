#[allow(dead_code)]
pub enum ConfigType {
    DEV,
    QA,
    STG,
    PROD
}

pub struct Config {
    config: String,
    runtime_config: String,
    credentials: String,
}

impl Config {

    pub fn new(config_type: ConfigType) -> Config {
        match config_type {
            ConfigType::DEV => {
                Config {
                    config: json!(
                        {
                            "read_host": "localhost",
                            "write_host": "localhost",
                            "port": 3306,
                            "db_name": "wallet"
                        }
                    ).to_string(),
                    runtime_config: json!(
                        {
                            "": ""
                        }
                    ).to_string(),
                    credentials: json!(
                        {
                            "user": "wallet",
                            "pass": "wallet"
                        }
                    ).to_string()
                }
            },
            ConfigType::QA => {
                Config {
                    config: json!(
                        {
                            "read_host": "wallet.cow5nksk7vuq.us-west-2.rds.amazonaws.com",
                            "write_host": "wallet.cow5nksk7vuq.us-west-2.rds.amazonaws.com",
                            "port": 3306,
                            "db_name": "wallet"
                        }
                    ).to_string(),
                    runtime_config: json!(
                        {
                            "": ""
                        }
                    ).to_string(),
                    credentials: json!(
                        {
                            "user": "root",
                            "pass": "NDyZBhCQNZgWLZKLNCnXwq3r"
                        }
                    ).to_string()
                }
            },
            ConfigType::STG => {
                Config {
                    config: json!(
                        {
                            "read_host": "localhost",
                            "write_host": "localhost",
                            "port": 3306,
                            "db_name": "wallet"
                        }
                    ).to_string(),
                    runtime_config: json!(
                        {
                            "": ""
                        }
                    ).to_string(),
                    credentials: json!(
                        {
                            "user": "wallet",
                            "pass": "wallet"
                        }
                    ).to_string()
                }
            },
            ConfigType:: PROD => {
                Config {
                    config: json!(
                        {
                            "read_host": "localhost",
                            "write_host": "localhost",
                            "port": 3306,
                            "db_name": "wallet"

                        }
                    ).to_string(),
                    runtime_config: json!(
                        {
                            "": ""
                        }
                    ).to_string(),
                    credentials: json!(
                        {
                            "user": "wallet",
                            "pass": "wallet"
                        }
                    ).to_string()
                }
            },
        }
    }

    pub fn get_config(&self) -> String {
        self.config.clone()
    }

    pub fn get_runtime_config(&self) -> String {
        self.runtime_config.clone()
    }

    pub fn get_credentials(&self) -> String {
        self.credentials.clone()
    }

}