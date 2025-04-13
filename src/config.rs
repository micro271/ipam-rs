use std::{env::var, net::IpAddr};

use axum::http::HeaderValue;
#[derive(Debug)]
pub struct Config {
    pub database: Database,
    pub app: Backend,
}

impl Config {
    pub fn init() -> Self {
        dotenv::dotenv().ok();

        Self {
            database: Database {
                name: var("DATABASE_NAME").expect("Database name not define"),
                port: var("DATABASE_PORT")
                    .expect("Port not defined")
                    .parse()
                    .expect("Invalid Port"),
                username: var("DATABASE_USER").expect("User database not defined"),
                password: var("DATABASE_PASSWD").expect("Password database not defined"),
                host: var("DATABASE_HOST").expect("Database Host not defined"),
            },
            app: Backend {
                port: var("APPLICATION_PORT")
                    .ok()
                    .filter(|x| !x.is_empty())
                    .map_or(3000, |x| x.parse().expect("Invalid port for application")),
                ip: var("APPLICATION_IP")
                    .ok()
                    .filter(|x| !x.is_empty())
                    .map_or("0.0.0.0".parse().unwrap(), |x| {
                        x.parse().expect("Invalid ip to backend")
                    }),
                allow_origin: var("ALLOW_ORIGIN")
                    .ok()
                    .filter(|x| !x.is_empty() && !x.eq("*"))
                    .map(|x| {
                        x.split_whitespace()
                            .filter_map(|x| x.parse::<HeaderValue>().ok())
                            .collect()
                    }),
            },
        }
    }
}

#[derive(Debug)]
pub struct Database {
    pub name: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
}

#[derive(Debug)]
pub struct Backend {
    pub port: u16,
    pub ip: IpAddr,
    pub allow_origin: Option<Vec<HeaderValue>>,
}
