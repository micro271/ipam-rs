use std::net::IpAddr;

use ipnet::IpNet;

use crate::models::device::{Device, UpdateDevice};

pub fn update_all_ip(mut devices_db: Vec<Device>, network: IpNet) -> Result<Vec<(Option<IpAddr>, Option<IpAddr>)>, error::Error> {
    use error::Error::*;

    if network.addr().is_ipv6() {
        return Err(Ipv6);
    }

    let len = (1 << (32 - network.prefix_len())) - 2;
    if devices_db.len() > (1 << (32 - network.prefix_len())) - 2 {
        devices_db.truncate(len);
    }

    let mut resp = Vec::new();
    let hosts = network.hosts().collect::<Vec<IpAddr>>();

    for (i, d) in hosts.into_iter().enumerate() {
        let tmp = devices_db.iter().find(|x| {

            match (&d, &x.ip) {
                (IpAddr::V4(new), IpAddr::V4(old)) => {
                    let pref = 3-i/256;
                    let tmp = &new.octets()[pref..];
                    let tmp_2 = &old.octets()[pref..];
                    if tmp == tmp_2 {
                        true
                    } else {
                        false
                    }
                },
                (_, _) => false
            }
            
        });

        
        if let Some(e) = tmp {
            resp.push((Some(e.ip), Some(d)));
        } else {
            resp.push((None, Some(d)));
        }
    }
    

    Ok(resp)
}

pub fn create_devices() {

}

pub mod error {
    #[derive(Debug)]
    pub enum Error {
        ManyDevices,
        Ipv6,
    }
}