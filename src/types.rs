pub struct SessionDetails {
    pub id: i64,
    pub realm: String,
    pub authid: String,
    pub auth_role: String,
    pub static_serializer: bool,
}

impl SessionDetails {
    pub fn new(id: i64, realm: String, authid: String, auth_role: String, static_serializer: bool) -> Self {
        SessionDetails {
            id,
            realm,
            authid,
            auth_role,
            static_serializer,
        }
    }
}
