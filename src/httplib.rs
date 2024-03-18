use {
    crate::{
        structs::{HTTPFilters, HttpData, LibOptions},
        utils,
    },
    async_recursion::async_recursion,
    futures::stream::StreamExt,
    rand::{distributions::Alphanumeric, thread_rng as rng, Rng},
    reqwest::{
        header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
        redirect::Policy,
        Client, Response, Url,
    },
    scraper::{Html, Selector},
    std::collections::{HashMap, HashSet},
};

#[allow(clippy::too_many_arguments)]
#[async_recursion]
#[must_use]
pub async fn return_http_data(options: &LibOptions) -> HashMap<String, HttpData> {
    let threads = if options.hosts.len() < options.threads {
        options.hosts.len()
    } else {
        options.threads
    };

    futures::stream::iter(options.hosts.clone().into_iter().map(|host| {
        // Use a random user agent
        let user_agent = utils::return_random_string(&options.user_agents);

        // Create futures
        let https_send_fut = options
            .client
            .get(format!("https://{host}"))
            .header(USER_AGENT, &user_agent);

        let http_send_fut = options
            .client
            .get(format!("http://{host}"))
            .header(USER_AGENT, &user_agent);

        let mut http_data = HttpData::default();

        let mut response = Option::<Response>::None;

        async move {
            if options.retries > 1 {
                for _ in 0..options.retries {
                    if let Some(resp) = https_send_fut.try_clone() {
                        if let Ok(resp) = resp.send().await {
                            response = Some(resp);
                            break;
                        }
                    } else if let Some(resp) = http_send_fut.try_clone() {
                        if let Ok(resp) = resp.send().await {
                            response = Some(resp);
                            break;
                        }
                    }
                }
            } else if let Ok(resp) = https_send_fut.send().await {
                response = Some(resp);
            } else if let Ok(resp) = http_send_fut.send().await {
                response = Some(resp);
            }

            match response {
                Some(resp) => {
                    http_data.host_url = return_url(resp.url().clone());
                    if options.assign_response_data {
                        http_data =
                            assign_response_data(http_data, resp, options.return_filters).await;
                    } else {
                        http_data.status_code = resp.status().as_u16();
                        http_data.http_status = "ACTIVE".to_string();
                    };
                }
                None => {
                    http_data.http_status = "INACTIVE".to_string();
                }
            };

            if !options.quiet_flag
                && (!http_data.host_url.is_empty() && options.conditional_response_code == 0)
                || ((!http_data.host_url.is_empty() && options.conditional_response_code != 0)
                    && (http_data.status_code >= options.conditional_response_code
                        && http_data.status_code <= options.conditional_response_code + 99))
            {
                if options.show_status_codes {
                    println!("{},{}", http_data.host_url, http_data.status_code);
                } else {
                    println!("{}", http_data.host_url);
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
        .trust_dns(true)
        .use_native_tls()
        .build()
        .expect("Failed to create HTTP client")
}

#[must_use]
pub fn return_url(mut url: Url) -> String {
    url.set_path("");
    url.set_query(None);
    url.to_string()
}

#[allow(clippy::field_reassign_with_default)]
pub async fn assign_response_data(
    mut http_data: HttpData,
    resp: Response,
    return_filers: bool,
) -> HttpData {
    let headers = resp.headers().clone();
    let url = resp.url().clone();

    http_data.http_status = "ACTIVE".to_string();
    http_data.status_code = resp.status().as_u16();

    http_data.final_url = url.to_string();
    http_data.protocol = resp.url().scheme().to_string();
    http_data.content_type = if headers.contains_key(CONTENT_TYPE) {
        headers[CONTENT_TYPE]
            .to_str()
            .unwrap_or_default()
            .to_string()
    } else {
        String::new()
    };

    http_data.headers = format!("{headers:?}");

    let full_body = resp.text().await.unwrap_or_default();

    http_data.content_length = if headers.contains_key(CONTENT_LENGTH) {
        headers[CONTENT_LENGTH]
            .to_str()
            .unwrap_or_default()
            .parse()
            .unwrap_or_default()
    } else {
        full_body.chars().count() as u64
    };

    return_title_and_body(&mut http_data, &full_body);

    http_data.words_count = full_body.split(' ').count();
    http_data.lines = full_body.lines().count() + 1;

    if return_filers {
        let host = url.host_str().unwrap_or_default();
        let client = return_http_client(5, 3);
        let user_agents = utils::user_agents();
        http_data.bad_data = return_filters_data(host, client, user_agents).await;
    }

    http_data
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
        assign_response_data: true,
        quiet_flag: true,
        ..Default::default()
    };

    let data = return_http_data(&lib_options).await;

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
