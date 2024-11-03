use super::*;
use ipnet::IpNet;

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateNetwork {
    pub network: Option<IpNet>,
    pub description: Option<String>,
    pub available: Option<u32>,
    pub used: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub id: Uuid,
    pub vlan: Option<Vlan>,
    pub network: IpNet,
    pub description: Option<String>,
    pub available: u32,
    pub used: u32,
    pub total: u32,
}

#[derive(Debug)]
pub struct Vlan(u16);

impl Vlan {
    pub fn set_vlan(&mut self, vlan: i32) -> Result<(), VlanError> {
        if vlan < 1 {
            Err(VlanError::Invalid)
        } else if vlan > 4096 {
            Err(VlanError::Exeded)
        } else {
            **self = vlan as u16;
            Ok(())
        }
    }

    pub fn new(vlan: i32) -> Result<Self, VlanError> {
        let mut v = Self(0);
        v.set_vlan(vlan)?;
        Ok(v)
    }
}

#[derive(Debug)]
pub enum VlanError {
    Invalid,
    Exeded,
}

impl Serialize for Vlan {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.0)
    }
}

impl<'de> Deserialize<'de> for Vlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(u16::deserialize(deserializer)?))
    }
}

impl std::ops::Deref for Vlan {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Vlan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
