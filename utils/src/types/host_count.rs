use ipnet::IpNet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, sqlx::Type, Default)]
#[sqlx(transparent)]
#[must_use]
pub struct HostCount(i32);

pub enum Operation {
    Add(i32),
    Sub(i32),
}

impl HostCount {
    const MAX: i32 = 0x00FF_FFFF;
    const MAX_BITS: u8 = 24;

    #[must_use]
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }

    /// # Errors
    ///
    /// Will return `Err` if:
    ///     - The IP's bits are greater then prefix
    ///     - The IP's bits or the prefix's bits are greater than 128
    ///     - The host's number is greater than `HostCount::MAX`
    ///     - The type parameter `T` fails the parse to u8
    #[must_use]
    pub fn new_with_operation<T>(bits: T, prefix: T, op: &Operation) -> Result<Self, HostCountError>
    where
        T: TryInto<u8>,
    {
        let bits: u8 = bits
            .try_into()
            .map_err(|_| HostCountError::ParseOutOfRange)
            .and_then(|x| (x <= 128).then_some(x).ok_or(HostCountError::BitsOutRange))?;

        let prefix: u8 = prefix
            .try_into()
            .map_err(|_| HostCountError::ParseOutOfRange)
            .and_then(|x| (x <= 128).then_some(x).ok_or(HostCountError::BitsOutRange))?;

        let bits_hosts = bits
            .checked_sub(prefix)
            .filter(|x| Self::MAX_BITS.lt(x))
            .ok_or(HostCountError::BitsHostTooLarge)?
            .into();

        2i32.checked_pow(bits_hosts)
            .map(|x| if x > 2 { x - 2 } else { x })
            .and_then(|x| match op {
                Operation::Add(n) => x.checked_add(*n),
                Operation::Sub(n) => x.checked_sub(*n),
            });

        Ok(Self(0))
    }

    #[must_use]
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

    #[must_use]
    pub fn new_from_bits_with_sub(bits: u8, prefix: u8, sub: u32) -> Option<Self> {
        2u32.checked_pow(bits.checked_sub(prefix)? as u32)
            .map(|x| if x > 2 { x - 2 } else { x })?
            .checked_sub(sub)
            .map(|x| Self((x as i32).min(Self::MAX)))
    }

    #[must_use]
    pub fn new_from_ipnet_with_sub(ipnet: IpNet, sub: u32) -> Option<Self> {
        let bits = ipnet.max_prefix_len();
        let prefix = ipnet.prefix_len();

        Self::new_from_bits_with_sub(bits, prefix, sub)
    }

    pub fn new_max() -> Self {
        HostCount(Self::MAX)
    }

    #[must_use]
    pub fn max() -> u32 {
        Self::MAX as u32
    }

    #[must_use]
    pub fn is_max(&self) -> bool {
        self.0 == Self::MAX
    }
}

impl TryFrom<i32> for HostCount {
    type Error = HostCountError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        (..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
            .ok_or(HostCountError::ParseOutOfRange)
    }
}

impl From<IpNet> for HostCount {
    fn from(value: IpNet) -> Self {
        Self::new(value.max_prefix_len(), value.prefix_len()).unwrap()
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
pub enum HostCountError {
    ParseOutOfRange,
    PrefixLTBits,
    PrefixOutRange,
    BitsOutRange,
    BitsHostTooLarge,
}

impl std::error::Error for HostCountError {}

impl std::fmt::Display for HostCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostCountError::ParseOutOfRange => {
                write!(f, "The host's number is longer then {}", HostCount::max())
            }
            HostCountError::PrefixLTBits => write!(f, "Prefix is longer then bits"),
            HostCountError::BitsOutRange => write!(f, "Ip's bits is longer then 128 bits"),
            HostCountError::PrefixOutRange => write!(f, "Prefix is longer then 128 bits"),
            HostCountError::BitsHostTooLarge => write!(f, ""),
        }
    }
}

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
