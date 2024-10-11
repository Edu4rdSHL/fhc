use {
    clap::Parser,
    fhc::{args, httplib, structs::LibOptions, utils},
    std::collections::HashSet,
    tokio::io::{self, AsyncReadExt},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Eval args
    let args = args::Cli::parse();

    let filter_codes = args.filter_codes;
    let exclude_codes = args.exclude_codes;
    let threads = args.threads;
    let retries = args.retries;
    let timeout = args.timeout;
    let max_redirects = args.max_redirects;
    let user_agents_list = utils::user_agents();
    let show_status_codes = args.show_codes;

    let client = httplib::return_http_client(timeout, max_redirects);

    let mut buffer = String::new();

    let mut hosts = HashSet::new();

    if args.domain.is_some() {
        let domain = args.domain.expect("Error getting domain");
        if args.bruteforce {
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
        filter_codes,
        exclude_codes,
        show_status_codes,
        ..Default::default()
    };

    if !args.quiet {
        if show_status_codes {
            println!("DOMAIN,[FINAL_URL],[STATUS_CODE]");
        } else {
            println!("DOMAIN,[FINAL_URL]");
        }
    }

    let _ = httplib::return_http_data(&lib_options).await;

    Ok(())
}
