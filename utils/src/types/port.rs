use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type)]
pub struct Port(u16);

impl std::ops::Deref for Port {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Port {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Port {
    #[must_use]
    pub fn new(port: u16) -> Self {
        Port(port)
    }
}

impl std::cmp::PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::cmp::PartialEq<u16> for Port {
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

impl std::cmp::PartialEq<Port> for u16 {
    fn eq(&self, other: &Port) -> bool {
        *self == other.0
    }
}

#[cfg(test)]
mod test {
    use super::Port;

    #[test]
    fn eq_port_left_side() {
        let port = Port::new(10);
        assert!(10 == port);
    }

    #[test]
    fn eq_port_right_side() {
        let port = Port::new(10);
        assert!(port == 10);
    }
}
