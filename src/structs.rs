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
    pub checked_host: String,
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
    pub points_to_another_host: bool,
}

#[derive(Clone, Debug, Default)]
pub struct LibOptions {
    pub hosts: HashSet<String>,
    pub client: reqwest::Client,
    pub user_agents: Vec<String>,
    pub retries: usize,
    pub threads: usize,
    pub return_filters: bool,
    pub filter_codes: Option<String>,
    pub exclude_codes: Option<String>,
    pub show_full_data: bool,
    pub quiet_flag: bool,
}
