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
        client: httplib::return_http_client(args.timeout, args.max_redirects),
        user_agents: utils::user_agents(),
        retries: args.retries,
        threads: args.threads,
        filter_codes: args.filter_codes,
        exclude_codes: args.exclude_codes,
        show_full_data: args.show_full_data,
        ..Default::default()
    };

    if !args.quiet && args.show_full_data {
        println!("DOMAIN,[FINAL_URL],[STATUS_CODE]");
    }

    let _ = httplib::return_http_data(&lib_options, true).await;

    Ok(())
}
