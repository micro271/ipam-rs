use std::net::{IpAddr, Ipv4Addr};

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use ipnet::IpNet;

#[derive(Debug)]
pub struct SubnettingError(String);

pub struct BatchSubnet {
    iter: SubnetList,
    window: usize,
}

impl std::iter::Iterator for BatchSubnet {
    type Item = Vec<IpNet>;

    fn next(&mut self) -> Option<Self::Item> {
        let aux = self.iter.by_ref().take(self.window).collect::<Vec<_>>();

        (!aux.is_empty()).then_some(aux)
    }
}

#[derive(Debug)]
pub struct SubnetList {
    start: u32,
    end: u32,
    prefix: u8,
    hosts: u32,
    step: u32,
}

impl SubnetList {
    pub fn new(network: IpNet, prefix: u8) -> Result<Self, SubnettingError> {
        let network_prefix = network.prefix_len();

        if prefix <= network_prefix {
            return Err(SubnettingError(format!(
                "The prefix subnet {} is smaller than {}",
                prefix, network_prefix
            )));
        }

        let start = match network.network() {
            IpAddr::V4(ipv4_net) => u32::from(ipv4_net),
            IpAddr::V6(_) => {
                return Err(SubnettingError(
                    "You cannot create subnet in ipv6".to_string(),
                ));
            }
        };

        let subnets = 2u32.pow((prefix - network_prefix) as u32);

        let hosts = 2_u32.pow((32 - prefix) as u32);

        let end = start + (hosts * (subnets));
        Ok(Self {
            start,
            end,
            prefix,
            hosts,
            step: 0,
        })
    }
}

impl Iterator for SubnetList {
    type Item = IpNet;

    fn next(&mut self) -> Option<Self::Item> {
        ((self.start + (self.step * self.hosts)) <= self.end)
            .then(|| {
                let resp = IpNet::new(
                    IpAddr::V4(Ipv4Addr::from(self.start + (self.hosts * self.step))),
                    self.prefix,
                )
                .ok();

                self.step += 1;

                resp
            })
            .flatten()
    }
}

impl ExactSizeIterator for SubnetList {
    fn len(&self) -> usize {
        ((self.end - self.start) / self.hosts) as usize
    }
}

impl std::fmt::Display for SubnettingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subnneting error: {}", self.0)
    }
}

impl std::error::Error for SubnettingError {}

pub async fn ping(ip: IpAddr, timeout_ms: u64) -> Ping {
    let ip = ip.to_string();
    let duration = std::time::Duration::from_millis(timeout_ms)
        .as_secs_f32()
        .to_string();
    let ping = tokio::process::Command::new("ping")
        .args(["-W", &duration, "-c", "1", &ip])
        .output()
        .await;

    match ping {
        Ok(e) if e.status.code().unwrap_or(1) == 0 => Ping::Pong,
        _ => Ping::Fail,
    }
}

#[derive(Debug, PartialEq, PartialOrd, serde::Serialize)]
pub enum Ping {
    Pong,
    Fail,
}
impl std::fmt::Display for Ping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pong => write!(f, "Pong"),
            Self::Fail => write!(f, "Fail"),
        }
    }
}
impl IntoResponse for Ping {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .status(StatusCode::OK)
            .body(
                serde_json::json!({
                    "status": 200,
                    "ping": self.to_string()
                })
                .to_string(),
            )
            .unwrap_or_default()
            .into_response()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::LazyLock;
    use tokio::runtime::Runtime;

    static RUNTIME: LazyLock<Runtime> = std::sync::LazyLock::new(|| Runtime::new().unwrap());

    #[test]
    fn sub_net_first_prefix_fifty_six() {
        let ip = "192.168.0.1/24".parse::<IpNet>().unwrap();
        let subnet: SubnetList = (ip, 26).try_into().unwrap();
        let subnet = subnet.collect::<Vec<IpNet>>();
        let mut ip_result = Vec::new();
        ip_result.push("192.168.0.0/26".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.64/26".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.128/26".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.192/26".parse::<IpNet>().unwrap());
        assert!(subnet.contains(&ip_result[0]));
        assert!(subnet.contains(&ip_result[1]));
        assert!(subnet.contains(&ip_result[2]));
        assert!(subnet.contains(&ip_result[3]));
        assert!(subnet.len() == 4)
    }

    #[test]
    fn sub_net_first_prefix_fifty_eight() {
        let ip = "192.168.0.1/24".parse::<IpNet>().unwrap();
        let subnet: SubnetList = (ip, 28).try_into().unwrap();
        let subnet = subnet.collect::<Vec<IpNet>>();
        let mut ip_result = Vec::new();
        ip_result.push("192.168.0.0/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.16/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.32/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.48/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.64/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.80/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.96/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.112/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.128/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.144/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.160/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.176/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.192/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.208/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.224/28".parse::<IpNet>().unwrap());
        ip_result.push("192.168.0.240/28".parse::<IpNet>().unwrap());
        assert!(subnet.contains(&ip_result[0]));
        assert!(subnet.contains(&ip_result[1]));
        assert!(subnet.contains(&ip_result[2]));
        assert!(subnet.contains(&ip_result[3]));
        assert!(subnet.contains(&ip_result[4]));
        assert!(subnet.contains(&ip_result[5]));
        assert!(subnet.contains(&ip_result[6]));
        assert!(subnet.contains(&ip_result[7]));
        assert!(subnet.contains(&ip_result[8]));
        assert!(subnet.contains(&ip_result[9]));
        assert!(subnet.contains(&ip_result[10]));
        assert!(subnet.contains(&ip_result[11]));
        assert!(subnet.contains(&ip_result[12]));
        assert!(subnet.contains(&ip_result[13]));
        assert!(subnet.contains(&ip_result[14]));
        assert!(subnet.contains(&ip_result[15]));
        assert!(subnet.len() == 16);
    }

    #[test]
    fn sub_net_first_prefix_fifty_four_above_twenty_one() {
        let ip = "192.168.0.1/16".parse::<IpNet>().unwrap();
        let subnet: SubnetList = (ip, 28).try_into().unwrap();
        assert!(subnet.len() == 4096);
    }

    #[test]
    fn sub_net_first_prefix_fifteen_above_twenty_four() {
        let ip = "192.168.0.1/15".parse::<IpNet>().unwrap();
        let subnet: SubnetList = (ip, 24).try_into().unwrap();
        assert!(subnet.len() == 512);
    }

    #[test]
    fn ping_test_pong() {
        let resp = RUNTIME.block_on(async { ping("192.168.0.1".parse().unwrap(), 100).await });
        assert_eq!(Ping::Pong, resp);
    }

    #[test]
    fn ping_test_fail() {
        let resp = RUNTIME.block_on(async { ping("192.168.1.50".parse().unwrap(), 100).await });
        assert_eq!(Ping::Fail, resp);
    }
}
