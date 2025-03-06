use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct VlanId(i16);

impl VlanId {
    pub const MAX: i16 = 0x0FFF;

    pub fn new(value: i16) -> Result<Self, OutOfRange> {
        value.try_into()
    }

    pub fn set_vlan(&mut self, id: i16) -> Result<(), OutOfRange> {
        if (2..=Self::MAX).contains(&id) {
            self.0 = id;
            Ok(())
        } else {
            Err(OutOfRange)
        }
    }
}

impl std::cmp::PartialEq for VlanId {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl std::cmp::PartialEq<i16> for VlanId {
    fn eq(&self, other: &i16) -> bool {
        self.0.eq(other)
    }
}

impl TryFrom<i16> for VlanId {
    type Error = OutOfRange;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        if !(2..=Self::MAX).contains(&value) {
            Err(OutOfRange)
        } else {
            Ok(Self(value))
        }
    }
}

impl std::default::Default for VlanId {
    fn default() -> Self {
        Self(1)
    }
}

impl std::ops::Deref for VlanId {
    type Target = i16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for VlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct OutOfRange;

impl std::fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Out of range")
    }
}
impl std::error::Error for OutOfRange {}

#[cfg(test)]
mod test {
    use crate::type_net::vlan::VlanId;

    #[test]
    fn vlan_negative_error() {
        let vlan = VlanId::new(-1);
        assert!(vlan.is_err());
    }

    #[test]
    fn vlan_out_range_error() {
        let vlan = VlanId::new(4096);
        assert!(vlan.is_err());
    }

    #[test]
    fn vlan_ok() {
        let vlan = VlanId::new(4095);
        assert!(vlan.is_ok());
    }

    #[test]
    fn vlan_cmp_with_vlan_eq_false() {
        let one = VlanId::new(4095).unwrap();
        let two = VlanId::new(1094).unwrap();
        assert_eq!(one == two, false);
    }

    #[test]
    fn vlan_cmp_with_vlan_eq_true() {
        let one = VlanId::new(4095).unwrap();
        let two = VlanId::new(4095).unwrap();
        assert!(one == two);
    }

    #[test]
    fn vlan_cmp_with_i16_eq_true() {
        let one = VlanId::new(4095).unwrap();
        assert!(one == 4095);
    }

    #[test]
    fn vlan_cmp_with_i16_eq_false() {
        let one = VlanId::new(4095).unwrap();
        assert_eq!(one == 5, false);
    }

    #[test]
    fn vlan_deref_cmp_with_i16_eq_false() {
        let one = VlanId::new(4095).unwrap();
        assert_eq!(*one == 4, false);
    }

    #[test]
    fn vlan_deref_cmp_with_i16_eq_true() {
        let one = VlanId::new(4095).unwrap();
        assert!(*one == 4095);
    }
}
