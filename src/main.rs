use {
    clap::{value_t, App, Arg},
    fhc::{httplib, structs::LibOptions, utils},
    std::collections::HashSet,
    tokio::io::{self, AsyncReadExt},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Eval args
    let matches = App::new("FHC")
        .version(clap::crate_version!())
        .author("Eduard Tolosa <edu4rdshl@protonmail.com>")
        .about("Fast HTTP Checker.")
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .takes_value(true)
                .help("Number of threads. Default: 50"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .help("Timeout in seconds. Default: 3"),
        )
        .arg(
            Arg::with_name("show-codes")
                .short("s")
                .long("show-codes")
                .takes_value(false)
                .help("Show status codes for discovered hosts."),
        )
        .arg(
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .takes_value(true)
                .help("Target domain. When it's specified, a wordlist can be used from stdin for bruteforcing."),
        )
        .arg(
            Arg::with_name("retries")
                .short("r")
                .long("retries")
                .takes_value(true)
                .help("Max number of http probes per target."),
        ).arg(
            Arg::with_name("max-redirects")
                .short("L")
                .long("max-redirects")
                .takes_value(true)
                .help("Max number of redirects. Default: 0"),
        )
        .arg(
            Arg::with_name("bruteforce")
                .short("b")
                .long("bruteforce")
                .takes_value(false)
                .help("Bruteforce subdomains."),
        )
        .arg(
            Arg::with_name("1xx")
                .short("1")
                .long("1xx")
                .takes_value(false)
                .help("Show URLs with 100-199 response codes only."),
        )
        .arg(
            Arg::with_name("2xx")
                .short("2")
                .long("2xx")
                .takes_value(false)
                .help("Show URLs with 200-299 response codes only."),
        )
        .arg(
            Arg::with_name("3xx")
                .short("3")
                .long("3xx")
                .takes_value(false)
                .help("Show URLs with 300-399 response codes only."),
        )
        .arg(
            Arg::with_name("4xx")
                .short("4")
                .long("4xx")
                .takes_value(false)
                .help("Show URLs with 400-499 response codes only."),
        )
        .arg(
            Arg::with_name("5xx")
                .short("5")
                .long("5xx")
                .takes_value(false)
                .help("Show URLs with 500-599 response codes only."),
        )
        .get_matches();

    // Assign values or use defaults
    let conditional_response_code = if matches.is_present("1xx") {
        100
    } else if matches.is_present("2xx") {
        200
    } else if matches.is_present("3xx") {
        300
    } else if matches.is_present("4xx") {
        400
    } else if matches.is_present("5xx") {
        500
    } else {
        0
    };
    let threads = value_t!(matches.value_of("threads"), usize).unwrap_or_else(|_| 50);
    let retries = value_t!(matches.value_of("retries"), usize).unwrap_or_else(|_| 1);
    let timeout = value_t!(matches.value_of("timeout"), u64).unwrap_or_else(|_| 3);
    let max_redirects = value_t!(matches.value_of("max-redirects"), usize).unwrap_or_else(|_| 0);
    let user_agents_list = utils::user_agents();
    let show_status_codes = matches.is_present("show-codes");

    let client = httplib::return_http_client(timeout, max_redirects);

    let mut buffer = String::new();

    let mut hosts = HashSet::new();

    if matches.is_present("domain") {
        let domain = value_t!(matches, "domain", String).unwrap();
        if matches.is_present("bruteforce") {
            io::stdin().read_to_string(&mut buffer).await?;
            hosts = buffer
                .lines()
                .map(|word| format!("{word}.{domain}"))
                .collect();
        } else {
            hosts.insert(domain);
        }
    } else {
        io::stdin().read_to_string(&mut buffer).await?;
        hosts = buffer.lines().map(str::to_owned).collect();
    };

    let lib_options = LibOptions {
        hosts,
        client,
        user_agents: user_agents_list,
        retries,
        threads,
        conditional_response_code,
        show_status_codes,
        ..Default::default()
    };

    httplib::return_http_data(&lib_options).await;

    Ok(())
}
