use ipnet::IpNet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, sqlx::Type, Default)]
#[sqlx(transparent)]
#[must_use]
pub struct HostCount(i32);

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Add(i32),
    Sub(i32),
    Any,
}

#[inline]
fn validate_u8<T: TryInto<u8>>(num_to_validate: T) -> Result<u8, HostCountError> {
    num_to_validate
        .try_into()
        .map_err(|_| HostCountError::ParseOutOfRange)
        .and_then(|x| {
            (x <= HostCount::MAX_BITS_LENGTH)
                .then_some(x)
                .ok_or(HostCountError::BitsOutRange)
        })
}

impl HostCount {
    const MAX: i32 = 0x00FF_FFFF;
    const MAX_BITS_LENGTH: u8 = 128;

    #[must_use]
    pub fn as_i32(&self) -> i32 {
        self.0
    }

    /// # Errors
    ///
    /// Will return `Err` if:
    ///     - The IP's bits are greater then prefix
    ///     - The IP's bits or the prefix's bits are greater than 128
    ///     - The host's number is greater than `HostCount::MAX`
    ///     - The type parameter `T` fails the parse to u8
    pub fn new_with_operation<T>(bits: T, prefix: T, op: Operation) -> Result<Self, HostCountError>
    where
        T: TryInto<u8>,
    {
        let bits: u8 = validate_u8(bits)?;
        let prefix: u8 = validate_u8(prefix)?;

        2i32.checked_pow(u32::from(
            bits.checked_sub(prefix)
                .ok_or(HostCountError::PrefixLTBits)?,
        ))
        .ok_or(HostCountError::Overflow)
        .map(|x| if x > 2 { x - 2 } else { x })
        .and_then(|x| match op {
            Operation::Any => match x {
                Self::MAX.. => Err(HostCountError::Overflow),
                _ => Ok(Self(x)),
            },
            Operation::Add(n) => x
                .checked_add(n)
                .filter(|x| Self::MAX.ge(x))
                .map(Self)
                .ok_or(HostCountError::Overflow),
            Operation::Sub(n) => x
                .checked_sub(n)
                .filter(|x| 0.le(x))
                .map(Self)
                .ok_or(HostCountError::Underflow),
        })
    }

    #[must_use]
    pub fn new(bits: u8, prefix: u8) -> Option<Self> {
        Self::new_with_operation(bits, prefix, Operation::Any).ok()
    }

    #[must_use]
    pub fn new_from_bits_with_sub(bits: u8, prefix: u8, sub: i32) -> Option<Self> {
        match Self::new_with_operation(bits, prefix, Operation::Sub(sub)) {
            Ok(e) => Some(e),
            Err(HostCountError::Overflow) => Self::new_max().into(),
            Err(HostCountError::Underflow) => Self(0).into(),
            _ => None,
        }
    }

    #[must_use]
    pub fn new_from_ipnet_with_sub(ipnet: IpNet, sub: i32) -> Option<Self> {
        Self::new_from_bits_with_sub(ipnet.max_prefix_len(), ipnet.prefix_len(), sub)
    }

    pub fn new_max() -> Self {
        HostCount(Self::MAX)
    }

    #[must_use]
    pub fn max() -> i32 {
        Self::MAX
    }

    #[must_use]
    pub fn is_max(&self) -> bool {
        self.0 == Self::MAX
    }
}

impl TryFrom<i32> for HostCount {
    type Error = HostCountError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            ..0 => Err(HostCountError::Underflow),
            ovr if ovr > Self::MAX + 1 => Err(HostCountError::Overflow),
            e => Ok(Self(e)),
        }
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

impl std::ops::Add<i32> for HostCount {
    type Output = HostCount;
    fn add(self, rhs: i32) -> Self::Output {
        if rhs < 0 {
            self - (-rhs)
        } else {
            self.0
                .checked_add(rhs)
                .filter(|x| Self::MAX.ge(x))
                .map_or(Self::new_max(), Self)
        }
    }
}

impl std::ops::Add for HostCount {
    type Output = HostCount;
    fn add(self, rhs: Self) -> Self::Output {
        self + rhs.0
    }
}

impl std::ops::Sub<i32> for HostCount {
    type Output = HostCount;
    fn sub(self, rhs: i32) -> Self::Output {
        if rhs < 0 {
            self + (-rhs)
        } else {
            self.0
                .checked_sub(rhs)
                .filter(|x| 0.le(x))
                .map_or(Self(0), Self)
        }
    }
}

impl std::ops::Sub for HostCount {
    type Output = HostCount;
    fn sub(self, rhs: Self) -> Self::Output {
        self - rhs.0
    }
}

impl std::ops::Neg for HostCount {
    type Output = i32;
    fn neg(self) -> Self::Output {
        -self.0
    }
}

impl std::ops::AddAssign<i32> for HostCount {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

impl std::ops::AddAssign for HostCount {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::SubAssign<i32> for HostCount {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl std::ops::SubAssign for HostCount {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

#[derive(Debug, PartialEq)]
pub enum HostCountError {
    ParseOutOfRange,
    PrefixLTBits,
    PrefixOutRange,
    BitsOutRange,
    BitsHostTooLarge,
    Overflow,
    Underflow,
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
            HostCountError::Overflow => write!(f, ""),
            HostCountError::Underflow => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::host_count::{HostCount, HostCountError};
    use ipnet::IpNet;

    #[test]
    fn host_counter_instance_from_prefix() {
        let ipnet = "172.30.0.0/24".parse::<IpNet>().unwrap();
        let pref = HostCount::new(ipnet.max_prefix_len(), ipnet.prefix_len()).unwrap();
        assert_eq!(pref.as_i32(), 254);
    }

    #[test]
    fn host_counter_instance_from_prefix_31() {
        let ipnet = "172.30.0.0/31".parse::<IpNet>().unwrap();
        let pref = HostCount::new(ipnet.max_prefix_len(), ipnet.prefix_len()).unwrap();
        assert_eq!(pref.as_i32(), 2);
    }

    #[test]
    fn host_counter_instance_from_prefix_32() {
        let ipnet = "172.30.0.0/32".parse::<IpNet>().unwrap();
        let pref = HostCount::new(ipnet.max_prefix_len(), ipnet.prefix_len()).unwrap();
        assert_eq!(pref.as_i32(), 1);
    }

    #[test]
    fn host_counter_instance_from_i32() {
        assert_eq!(HostCount::try_from(10), Ok(HostCount(10)));
    }

    #[test]
    fn host_counter_addition_overflow() {
        let pref: HostCount = HostCount::MAX.try_into().unwrap();
        assert_eq!(pref.as_i32(), HostCount::MAX);
        assert_eq!(HostCount::MAX, pref.as_i32());
    }

    #[test]
    fn host_counter_add_hostcount() {
        let pref = HostCount::new_max();
        let pref = pref + (-HostCount::new_max());
        assert_ne!(pref, HostCount::new_max());
        assert_eq!(pref, HostCount(0));
    }

    #[test]
    fn host_counter_sub_hostcount_neg() {
        let pref = HostCount(0);
        let pref = pref - (-HostCount::new_max());
        assert_eq!(pref, HostCount::new_max());
        assert_ne!(pref, HostCount(0));
    }

    fn host_counter_sub_hostcount_() {
        let pref = HostCount(500000);
        let pref = pref - HostCount::new_max();
        assert_ne!(pref, HostCount::new_max());
        assert_eq!(pref, HostCount(0));
    }

    #[test]
    fn host_counter_sub_i32() {
        let pref = HostCount(0);
        let pref = pref - (-HostCount::MAX);
        assert_eq!(pref, HostCount::new_max());
        assert_ne!(pref, HostCount(0));
    }

    #[test]
    fn host_counter_sub_assign_i32() {
        let mut pref = HostCount(0);
        pref -= 500;
        assert_eq!(pref, HostCount(0));
    }

    #[test]
    fn host_counter_add_assign_i32() {
        let mut pref = HostCount(4000);
        pref += HostCount::max();
        assert_eq!(pref, HostCount::new_max());
    }
}
