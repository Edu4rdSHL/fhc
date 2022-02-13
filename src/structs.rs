#[derive(Clone)]
pub struct HttpData {
    pub http_status: String,
    pub host_url: String,
    pub status_code: u16,
}

impl Default for HttpData {
    fn default() -> Self {
        HttpData {
            http_status: String::from(""),
            host_url: String::from(""),
            status_code: 0,
        }
    }
}
