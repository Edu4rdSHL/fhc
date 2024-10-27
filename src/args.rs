use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long, default_value = "50")]
    /// Number of threads to use
    pub threads: usize,
    #[clap(long, default_value = "3")]
    /// Timeout in seconds
    pub timeout: u64,
    #[clap(short, long)]
    /// Show HTTP status codes, final URL and domain
    pub show_full_data: bool,
    #[clap(short, long)]
    /// Domain to check - can be omitted if using stdin
    pub domain: Option<String>,
    #[clap(short, long, default_value = "1")]
    /// Number of retries
    pub retries: usize,
    #[clap(short = 'L', long, default_value = "10")]
    /// Maximum number of redirects
    pub max_redirects: usize,
    #[clap(short, long)]
    /// Enable bruteforce mode
    pub bruteforce: bool,
    #[clap(short, long)]
    /// Filter status codes. A comma separated list can be used
    pub filter_codes: Option<String>,
    #[clap(short, long)]
    /// Exclude status codes. A comma separated list can be used
    pub exclude_codes: Option<String>,
    #[clap(short, long)]
    /// Quiet mode. This will suppress all fancy output except for the final results
    pub quiet: bool,
}
