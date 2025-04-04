use ipnet::IpNet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct HostCount(i32);

impl HostCount {
    pub const MAX: i32 = 0x00FFFFFF;

    pub fn new(bits: u8, prefix: u8) -> Option<Self> {
        2_i32
            .checked_pow(bits.checked_sub(prefix)? as u32)
            .map(|x| {
                Self({
                    if x > 2 {
                        (x - 2).min(Self::MAX)
                    } else {
                        x.max(0)
                    }
                })
            })
    }

    pub fn new_from_bits_with_sub(bits: u8, prefix: u8, sub: u32) -> Option<Self> {
        (bits > prefix)
            .then(|| {
                2u32.pow((bits - prefix) as u32)
                    .checked_sub(sub)
                    .or(Some(0))
                    .map(|x| Self((x as i32).min(Self::MAX)))
            })
            .flatten()
    }

    pub fn new_from_ipnet_with_sub(ipnet: IpNet, sub: u32) -> Option<Self> {
        let bits = ipnet.max_prefix_len();
        let prefix = ipnet.prefix_len();

        Self::new_from_bits_with_sub(bits, prefix, sub)
    }

    pub fn new_from_bits_with_add(bits: u8, prefix: u8, sub: u32) -> Option<Self> {
        (bits > prefix)
            .then(|| {
                let avl = 2u32.pow((bits - prefix) as u32);

                avl.checked_add(sub)
                    .filter(|x| x < &avl)
                    .map(|x| Self((x as i32).min(Self::MAX)))
            })
            .flatten()
    }

    pub fn new_from_ipnet_with_add(ipnet: IpNet, sub: u32) -> Option<Self> {
        let bits = ipnet.max_prefix_len();
        let prefix = ipnet.prefix_len();

        Self::new_from_bits_with_add(bits, prefix, sub)
    }

    pub fn add(self, value: u32) -> Self {
        Self(
            value
                .try_into()
                .ok()
                .and_then(|x| self.0.checked_add(x).map(|x| x.min(Self::MAX)))
                .unwrap_or(Self::MAX),
        )
    }

    pub fn sub(self, value: u32) -> Self {
        Self(
            value
                .try_into()
                .ok()
                .and_then(|x| self.0.checked_sub(x).map(|x| x.max(0)))
                .unwrap_or(0),
        )
    }
}

impl TryFrom<i32> for HostCount {
    type Error = CountOfRange;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        (..=Self::MAX)
            .contains(&value)
            .then(|| Self(value))
            .ok_or(CountOfRange)
    }
}

impl From<IpNet> for HostCount {
    fn from(value: IpNet) -> Self {
        Self::new(value.max_prefix_len(), value.prefix_len()).unwrap()
    }
}

impl std::ops::Deref for HostCount {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::cmp::PartialEq for HostCount {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl std::cmp::PartialEq<i32> for HostCount {
    fn eq(&self, other: &i32) -> bool {
        self.0.eq(other)
    }
}

impl std::fmt::Display for HostCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct CountOfRange;

#[cfg(test)]
mod test {
    use crate::type_net::host_count::HostCount;
    use ipnet::IpNet;

    #[test]
    fn host_counter_instance_from_prefix() {
        let ipnet = "172.30.0.0/24".parse::<IpNet>().unwrap();
        let pref = HostCount::new(ipnet.max_prefix_len(), ipnet.prefix_len()).unwrap();
        assert_eq!(*pref, 254);
    }

    #[test]
    fn host_counter_instance_from_prefix_31() {
        let ipnet = "172.30.0.0/31".parse::<IpNet>().unwrap();
        let pref = HostCount::new(ipnet.max_prefix_len(), ipnet.prefix_len()).unwrap();
        assert_eq!(*pref, 2);
    }

    #[test]
    fn host_counter_instance_from_prefix_32() {
        let ipnet = "172.30.0.0/32".parse::<IpNet>().unwrap();
        let pref = HostCount::new(ipnet.max_prefix_len(), ipnet.prefix_len()).unwrap();
        assert_eq!(*pref, 1);
    }

    #[test]
    fn host_counter_instance_from_u32() {
        let pref: HostCount = 10.try_into().unwrap();
        assert_eq!(*pref, 10);
        assert_ne!(15, *pref);
    }

    #[test]
    fn host_counter_addition_is_err() {
        let pref: HostCount = 5000.try_into().unwrap();
        let resp = pref.add(HostCount::MAX.try_into().unwrap());
        assert_eq!(*pref, 5000);
        assert_eq!(*resp, HostCount::MAX as i32);
    }

    #[test]
    fn host_counter_addition_overflow() {
        let pref: HostCount = HostCount::MAX.try_into().unwrap();
        assert_eq!(*pref.add(20), HostCount::MAX as i32);
        assert_eq!(HostCount::MAX as i32, *pref);
    }
}
