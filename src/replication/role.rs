#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReplicationRole {
    Master,
    Slave
}

impl ToString for ReplicationRole {
    fn to_string(&self) -> String {
        match self {
            ReplicationRole::Master => "master".to_string(),
            ReplicationRole::Slave => "slave".to_string(),
        }
    }
}

impl ReplicationRole {
    pub fn needs_handshake(&self) -> bool {
        match self {
            ReplicationRole::Master => false,
            ReplicationRole::Slave => true,
        }
    }
}