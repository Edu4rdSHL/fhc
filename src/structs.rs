use std::collections::HashSet;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct HTTPFilters {
    pub bad_http_lengths: Vec<String>,
    pub bad_words_numbers: Vec<String>,
    pub bad_lines_numbers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct HttpData {
    pub http_status: String,
    pub status_code: u16,
    pub host_url: String,
    pub final_url: String,
    pub protocol: String,
    pub title: String,
    pub content_type: String,
    pub body: String,
    pub headers: String,
    pub content_length: u64,
    pub words_count: usize,
    pub lines: usize,
    pub bad_data: HTTPFilters,
    pub html_file_path: String,
    pub screenshot_data: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct LibOptions {
    pub hosts: HashSet<String>,
    pub client: reqwest::Client,
    pub user_agents: Vec<String>,
    pub retries: usize,
    pub threads: usize,
    pub return_filters: bool,
    pub conditional_response_code: u16,
    pub show_status_codes: bool,
    pub assign_response_data: bool,
    pub quiet_flag: bool,
}
