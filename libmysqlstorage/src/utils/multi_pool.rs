use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use mysql::{Pool, OptsBuilder, Opts};
use mysql::consts::CapabilityFlags;

#[derive(Deserialize)]
pub struct StorageConfig <'a> {
    pub read_host: &'a str,
    pub write_host: &'a str,
    pub port: u16,
    pub db_name: &'a str
}

#[derive(Deserialize)]
pub struct StorageCredentials <'a> {
    pub user: &'a str,
    pub pass: &'a str,
}

pub struct MultiPool {
    map: RwLock<HashMap<String, Arc<Pool>>>,
}

impl MultiPool {
    pub fn new() -> Self {
        MultiPool{map: RwLock::new(HashMap::new())}
    }

    pub fn get(&self, read_only: bool, config: &StorageConfig, credentials: &StorageCredentials) -> Option<Arc<Pool>> {

        let host_addr = if read_only {config.read_host} else {config.write_host};
        let connection_string = format!("{}:{}@{}:{}/{}", credentials.user, credentials.pass, host_addr, config.port, config.db_name);

        let mut c = match self.map.read() {
            Err(_) => None,
            Ok(ref map) => match map.get(&connection_string) {
                None => None,
                Some(x) => Some(x.clone()),
            }
        };

        if c.is_none() {

            let mut builder = OptsBuilder::default();

            builder.user(Some(credentials.user))
                   .pass(Some(credentials.pass))
                   .ip_or_hostname(Some(host_addr))
                   .db_name(Some(config.db_name))
                   .tcp_port(config.port)
                   .additional_capabilities(CapabilityFlags::CLIENT_FOUND_ROWS);

            let opts: Opts = builder.into();

            let pool = Pool::new_manual(1, 100, opts);
            if pool.is_err() {
                let err = pool.unwrap_err();
                error!("Error while connecting to the pool: {:?}", err);
                return None
            }
            let pool = Arc::new(pool.unwrap());
            self.map.write().unwrap().insert(connection_string.to_owned(), pool.clone());

            c = Some(pool)
        }

        c
    }
}