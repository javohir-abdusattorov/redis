pub mod commands {
    pub const PING: &'static str = "ping";
    pub const COMMAND: &'static str = "command";
    pub const ECHO: &'static str = "echo";
    pub const GET: &'static str = "get";
    pub const SET: &'static str = "set";
    pub const EXPIRE: &'static str = "expire";
    pub const DEL: &'static str = "del";
    pub const TTL: &'static str = "ttl";
    pub const KEYS: &'static str = "keys";
    pub const CONFIG: &'static str = "config";
    pub const INFO: &'static str = "info";
    pub const REPLCONF: &'static str = "replconf";
    pub const PSYNC: &'static str = "psync";
}

pub mod responses {
    pub const PONG: &'static str = "PONG";
    pub const OK: &'static str = "OK";
}