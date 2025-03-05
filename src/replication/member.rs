use itertools::Itertools;
use super::role::ReplicationRole;


#[derive(Clone)]
pub struct ReplicationMember {
    pub id: String,
    pub role: ReplicationRole,
    pub host: String,
    pub port: String,
}

impl ReplicationMember {
    pub fn new(role: ReplicationRole, id: String, address: String) -> Self {
        let (host, port) = address.split(":").next_tuple().unwrap();

        ReplicationMember {
            id,
            role,
            host: host.to_string(),
            port: port.to_string(),
        }
    }
}