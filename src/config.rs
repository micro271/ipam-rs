use std::{env::var, net::IpAddr};

use axum::http::HeaderValue;
#[derive(Debug)]
pub struct Config {
    pub database: Database,
    pub app: Backend,
}

impl Config {
    pub fn init() -> Result<Self, std::io::Error> {
        dotenv::dotenv().ok();

        Ok(Self {
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
                    .map(|x| x.parse().expect("Invalid port for application"))
                    .unwrap_or(3000),
                ip: var("APPLICATION_IP")
                    .ok()
                    .filter(|x| !x.is_empty())
                    .map(|x| x.parse().expect("Invalid ip to backend"))
                    .unwrap_or("0.0.0.0".parse().unwrap()),
                origin_allow: var("ORIGIN_ALLOW").ok().filter(|x| !x.is_empty()).map(|x| {
                    x.split_whitespace().map(|x| x.parse().unwrap()).collect::<Vec<HeaderValue>>()
                }).unwrap_or(Vec::from(["localhost".parse().unwrap()])),
            },
        })
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
    pub origin_allow: Vec<HeaderValue>,
}