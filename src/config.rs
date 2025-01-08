use serde::Deserialize;
use std::{env::var, net::IpAddr};
#[derive(Deserialize)]
pub struct Config {
    pub database: Database,
    pub app: Application,
}

impl Config {
    pub fn init() -> Result<Self, std::io::Error> {
        dotenv::dotenv().ok();

        Ok(Self {
            database: Database {
                name: var("DATABASE_NAME").expect("Database name not define"),
                port: var("PORT")
                    .expect("Port not defined")
                    .parse()
                    .expect("Invalid Port"),
                username: var("USER_DATABASE").expect("User database not defined"),
                password: var("PASSWORD_DATABASE").expect("Password database not defined"),
                host: var("DATABASE_HOST").expect("Database Host not defined"),
            },
            app: Application {
                port: var("APPLICATION_PORT")
                    .map(|x| x.parse().expect("Invalid port for application"))
                    .unwrap_or(3000),
                ip: var("APPLICATION_PORT")
                    .map(|x| x.parse().expect("Invalid ip"))
                    .unwrap_or("0.0.0.0".parse().unwrap()),
            },
        })
    }
}

#[derive(Deserialize)]
pub struct Database {
    pub name: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
}

#[derive(Deserialize)]
pub struct Application {
    pub port: u16,
    pub ip: IpAddr,
}
