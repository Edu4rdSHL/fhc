use std::collections::{HashMap, HashSet};

use async_recursion::async_recursion;
use rand::{distributions::Alphanumeric, thread_rng as rng, Rng};
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    redirect::Policy,
    Client, Response,
};

use crate::structs::{HTTPFilters, LibOptions};

use {
    crate::{structs::HttpData, utils},
    futures::stream::StreamExt,
    reqwest::{self, header::USER_AGENT},
    scraper::{Html, Selector},
};

#[allow(clippy::too_many_arguments)]
#[async_recursion]
pub async fn return_http_data(options: &LibOptions) -> HashMap<String, HttpData> {
    let threads = if options.hosts.len() < options.threads {
        options.hosts.len()
    } else {
        options.threads
    };

    futures::stream::iter(options.hosts.clone().into_iter().map(|host| {
        // Use a random user agent
        let user_agent = utils::return_random_string(&options.user_agents);
        // HTTP/HTTP URLs
        let https_url = format!("https://{}", host);
        let http_url = format!("http://{}", host);
        // Create futures
        let https_send_fut = options
            .client
            .get(&https_url)
            .header(USER_AGENT, &user_agent);
        let http_send_fut = options
            .client
            .get(&http_url)
            .header(USER_AGENT, &user_agent);

        let mut http_data = HttpData::default();

        let mut is_http = false;
        let mut response = Option::<Response>::None;

        async move {
            if options.retries != 1 {
                let mut counter = 0;
                while counter < options.retries {
                    if let Ok(resp) = https_send_fut
                        .try_clone()
                        .expect("Failed to clone https future")
                        .send()
                        .await
                    {
                        response = Some(resp);
                        break;
                    } else if let Ok(resp) = http_send_fut
                        .try_clone()
                        .expect("Failed to clone http future")
                        .send()
                        .await
                    {
                        is_http = true;
                        response = Some(resp);
                        break;
                    }
                    counter += 1
                }
            } else if let Ok(resp) = https_send_fut.send().await {
                response = Some(resp);
            } else if let Ok(resp) = http_send_fut.send().await {
                is_http = true;
                response = Some(resp);
            }

            match response {
                Some(resp) => {
                    if options.assign_response_data {
                        http_data =
                            assign_response_data(http_data, resp, options.return_filters).await;
                    } else {
                        http_data.host_url = if is_http { http_url } else { https_url.clone() };
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
                    println!("{},{}", http_data.host_url, http_data.status_code)
                } else {
                    println!("{}", http_data.host_url)
                }
            }
            (host, http_data)
        }
    }))
    .buffer_unordered(threads)
    .collect::<HashMap<String, HttpData>>()
    .await
}

pub fn return_http_client(timeout: u64, max_redirects: usize) -> Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .redirect(Policy::limited(max_redirects))
        .danger_accept_invalid_certs(true)
        .trust_dns(true)
        .use_native_tls()
        .build()
        .expect("Failed to create HTTP client")
}

#[allow(clippy::field_reassign_with_default)]
pub async fn assign_response_data(
    mut http_data: HttpData,
    resp: Response,
    return_filers: bool,
) -> HttpData {
    let headers = resp.headers().clone();
    let mut url = resp.url().to_owned();

    http_data.http_status = "ACTIVE".to_string();
    http_data.status_code = resp.status().as_u16();
    http_data.host_url = {
        url.set_path("");
        url.set_query(None);
        url.to_string()
    };
    http_data.final_url = url.to_string();
    http_data.protocol = resp.url().scheme().to_string();
    http_data.content_type = if headers.contains_key(CONTENT_TYPE) {
        headers[CONTENT_TYPE]
            .to_str()
            .unwrap_or_default()
            .to_string()
    } else {
        "".to_string()
    };

    http_data.headers = format!("{:?}", headers);

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

    http_data.body = return_body(&full_body).await;
    http_data.title = return_title(&full_body).await;
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

pub async fn return_title(data: &str) -> String {
    let document = Html::parse_document(data);
    match Selector::parse("title") {
        Ok(selector) => {
            if let Some(title_element) = document.select(&selector).next() {
                title_element.inner_html()
            } else {
                "NULL".to_string()
            }
        }
        Err(err) => {
            eprintln!("Failed to parse selector: {:?}", err);
            String::new()
        }
    }
}

pub async fn return_body(data: &str) -> String {
    let document = Html::parse_document(data);
    match Selector::parse("body") {
        Ok(selector) => {
            if let Some(body_element) = document.select(&selector).next() {
                body_element.inner_html()
            } else {
                "NULL".to_string()
            }
        }
        Err(err) => {
            eprintln!("Failed to parse selector: {:?}", err);
            String::new()
        }
    }
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

    let data = return_http_data(&lib_options).await;

    data.iter()
        .map(|(_, http_data)| {
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
