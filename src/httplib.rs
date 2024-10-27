use {
    crate::{
        structs::{HTTPFilters, HttpData, LibOptions},
        utils,
    },
    futures::stream::StreamExt,
    rand::{distributions::Alphanumeric, thread_rng as rng, Rng},
    reqwest::{
        header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
        redirect::Policy,
        Client, Response,
    },
    scraper::{Html, Selector},
    std::collections::{HashMap, HashSet},
};

#[must_use]
pub async fn return_http_data(options: &LibOptions, from_cli: bool) -> HashMap<String, HttpData> {
    let threads = options.hosts.len().min(options.threads);

    let filter_codes = options.filter_codes.as_deref().unwrap_or_default();
    let exclude_codes = options.exclude_codes.as_deref().unwrap_or_default();

    futures::stream::iter(options.hosts.clone().into_iter().map(|host| {
        let user_agent = utils::return_random_user_agent(&options.user_agents);

        async move {
            let mut http_data = HttpData {
                checked_host: host.clone(),
                ..Default::default()
            };
            let mut response = None;

            // Attempt both HTTPS and HTTP requests, retry if necessary
            for _ in 0..options.retries {
                let https_req = options
                    .client
                    .get(format!("https://{}", host))
                    .header(USER_AGENT, &user_agent)
                    .send();

                let http_req = options
                    .client
                    .get(format!("http://{}", host))
                    .header(USER_AGENT, &user_agent)
                    .send();

                response = https_req.await.or(http_req.await).ok();

                if response.is_some() {
                    break;
                }
            }

            if let Some(resp) = response {
                // Those are always set
                http_data.protocol = resp.url().scheme().to_string();
                http_data.status_code = resp.status().as_u16();
                http_data.final_url = resp.url().to_string();

                if !from_cli {
                    assign_response_data(&mut http_data, resp, options.return_filters).await;
                }
            } else {
                http_data.http_status = "INACTIVE".to_string();
            }

            if !options.quiet_flag
                && !http_data.final_url.is_empty()
                && (filter_codes.is_empty()
                    || filter_codes.contains(&http_data.status_code.to_string()))
                && (exclude_codes.is_empty()
                    || !exclude_codes.contains(&http_data.status_code.to_string()))
            {
                if options.show_full_data {
                    println!(
                        "{},[{}],[{}]",
                        http_data.checked_host, http_data.final_url, http_data.status_code
                    );
                } else {
                    println!("{}://{}", http_data.protocol, http_data.checked_host);
                }
            }
            (host, http_data)
        }
    }))
    .buffer_unordered(threads)
    .collect::<HashMap<String, HttpData>>()
    .await
}

#[must_use]
pub fn return_http_client(timeout: u64, max_redirects: usize) -> Client {
    let policy = if max_redirects == 0 {
        Policy::none()
    } else {
        Policy::limited(max_redirects)
    };

    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .redirect(policy)
        .danger_accept_invalid_certs(true)
        .use_native_tls()
        .build()
        .expect("Failed to create HTTP client")
}

#[allow(clippy::field_reassign_with_default)]
pub async fn assign_response_data(http_data: &mut HttpData, resp: Response, return_filters: bool) {
    let headers = resp.headers().clone();
    let url = resp.url().clone();

    http_data.http_status = "ACTIVE".to_string();
    http_data.content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let full_body = (resp.text().await).unwrap_or_default();

    http_data.content_length = headers
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok()?.parse().ok())
        .unwrap_or_else(|| full_body.chars().count() as u64);

    http_data.headers = format!("{headers:?}");

    return_title_and_body(http_data, &full_body);

    http_data.words_count = full_body.split_whitespace().count();
    http_data.lines = full_body.lines().count() + 1;
    http_data.points_to_another_host = url.host_str() != Some(&http_data.checked_host);

    if return_filters {
        let host = url.host_str().unwrap_or_default();
        let client = return_http_client(5, 3);
        let user_agents = utils::user_agents();
        http_data.bad_data = return_filters_data(host, client, user_agents).await;
    }
}

pub fn return_title_and_body(http_data: &mut HttpData, body: &str) {
    let document = Html::parse_document(body);

    // Return title
    match Selector::parse("title") {
        Ok(selector) => {
            if let Some(title_element) = document.select(&selector).next() {
                http_data.title = title_element.inner_html();
            } else {
                http_data.title = "NULL".to_string();
            }
        }
        Err(err) => {
            eprintln!("Failed to parse selector: {err:?}");
        }
    }

    // Return body
    match Selector::parse("body") {
        Ok(selector) => {
            if let Some(body_element) = document.select(&selector).next() {
                http_data.body = body_element.inner_html();
            } else {
                http_data.body = "NULL".to_string();
            }
        }
        Err(err) => {
            eprintln!("Failed to parse selector: {err:?}");
        }
    }

    drop(document);
}

pub async fn return_filters_data(
    host: &str,
    client: Client,
    user_agents_list: Vec<String>,
) -> HTTPFilters {
    let mut urls_to_check = HashSet::new();
    let random_str = rng()
        .sample_iter(Alphanumeric)
        .take(16)
        .map(char::from)
        .collect::<String>();
    let words = vec![
        "admin".to_string() + &random_str + "/",
        ".htaccess".to_string() + &random_str,
        random_str.to_string() + "/",
        random_str.to_string(),
    ];
    for word in words {
        urls_to_check.insert(format!("{}/{}", &host, word));
    }

    let threads = urls_to_check.len();
    let mut http_filters = HTTPFilters::default();

    let lib_options = LibOptions {
        hosts: urls_to_check,
        client,
        user_agents: user_agents_list,
        retries: 1,
        threads,
        quiet_flag: true,
        ..Default::default()
    };

    let data = return_http_data(&lib_options, false).await;

    data.values()
        .map(|http_data| {
            http_filters
                .bad_http_lengths
                .append(&mut vec![http_data.content_length.to_string()]);
            http_filters.bad_words_numbers.append(&mut vec![http_data
                .body
                .split(' ')
                .count()
                .to_string()]);
            http_filters
                .bad_lines_numbers
                .append(&mut vec![http_data.lines.to_string()]);
        })
        .for_each(drop);

    http_filters
}
