use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use mysql::{Pool, OptsBuilder, Opts};
use mysql::consts::CapabilityFlags;

#[derive(Deserialize)]
pub struct StorageConfig {
    read_host: String,
    write_host: String,
    port: u16,
    db_name: String
}

#[derive(Deserialize)]
pub struct StorageCredentials {
    user: String,
    pass: String,
}

pub struct MultiPool {
    map: RwLock<HashMap<String, Arc<Pool>>>,
}

impl MultiPool {
    pub fn new() -> Self {
        MultiPool{map: RwLock::new(HashMap::new())}
    }

    pub fn get(&self, read_only: bool, config: &StorageConfig, credentials: &StorageCredentials) -> Option<Arc<Pool>> {

        let connection_string = format!("mysql://{}:{}@{}:{}/{}", credentials.user, credentials.pass, config.write_host, config.port, config.db_name);

        let mut c = match self.map.read() {
            Err(_) => None,
            Ok(ref map) => match map.get(&connection_string) {
                None => None,
                Some(x) => Some(x.clone()),
            }
        };

        if c.is_none() {

            let addr = "0.0.0.0";
            let user = "root";
            let pwd = "h!Ka1h0ePte;";
            let port = 3306;
            let db_name = "wallet";

            let mut builder = OptsBuilder::default();

            builder.user(Some(user))
                   .pass(Some(pwd))
                   .ip_or_hostname(Some(addr))
                   .db_name(Some(db_name))
                   .tcp_port(port)
                   .additional_capabilities(CapabilityFlags::CLIENT_FOUND_ROWS);


            let opts: Opts = builder.into();

            let pool = Pool::new_manual(1, 100, opts);
            if pool.is_err() {
                return None
            }
            let pool = Arc::new(pool.unwrap());
            self.map.write().unwrap().insert(connection_string.to_owned(), pool.clone());

            c = Some(pool)
        }

        c
    }
}