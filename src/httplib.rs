use std::collections::{HashMap, HashSet};

use reqwest::Client;

use {
    crate::{structs::HttpData, utils},
    futures::stream::StreamExt,
    reqwest::{self, header::USER_AGENT},
};

#[allow(clippy::too_many_arguments)]
pub async fn return_http_data(
    hosts: HashSet<String>,
    client: Client,
    user_agents_list: Vec<String>,
    retries: usize,
    mut threads: usize,
    conditional_response_code: u16,
    show_status_codes: bool,
    quiet_flag: bool,
) -> HashMap<String, HttpData> {
    if hosts.len() < threads {
        threads = hosts.len();
    }

    futures::stream::iter(hosts.into_iter().map(|host| {
        // Use a random user agent
        let user_agent = utils::return_random_string(&user_agents_list);
        // HTTP/HTTP URLs
        let https_url = format!("https://{}", host);
        let http_url = format!("http://{}", host);
        // Create futures
        let https_send_fut = client.get(&https_url).header(USER_AGENT, &user_agent);
        let http_send_fut = client.get(&http_url).header(USER_AGENT, &user_agent);

        async move {
            let mut http_data = HttpData::default();

            if retries != 1 {
                let mut counter = 0;
                while counter < retries {
                    if let Ok(resp) = https_send_fut
                        .try_clone()
                        .expect("Failed to clone https future")
                        .send()
                        .await
                    {
                        http_data.host_url = https_url.clone();
                        http_data.status_code = resp.status().as_u16();
                        http_data.http_status = "ACTIVE".to_string();
                        drop(resp)
                    } else if let Ok(resp) = http_send_fut
                        .try_clone()
                        .expect("Failed to clone http future")
                        .send()
                        .await
                    {
                        http_data.host_url = http_url.clone();
                        http_data.status_code = resp.status().as_u16();
                        http_data.http_status = "ACTIVE".to_string();
                        drop(resp)
                    }
                    counter += 1
                }
            } else if let Ok(resp) = https_send_fut.send().await {
                http_data.host_url = https_url;
                http_data.status_code = resp.status().as_u16();
                http_data.http_status = "ACTIVE".to_string();
                drop(resp)
            } else if let Ok(resp) = http_send_fut.send().await {
                http_data.host_url = http_url;
                http_data.status_code = resp.status().as_u16();
                http_data.http_status = "ACTIVE".to_string();
                drop(resp)
            } else {
                http_data.http_status = "INACTIVE".to_string();
            }
            if !quiet_flag && (!http_data.host_url.is_empty() && conditional_response_code == 0)
                || ((!http_data.host_url.is_empty() && conditional_response_code != 0)
                    && (http_data.status_code >= conditional_response_code
                        && http_data.status_code <= conditional_response_code + 99))
            {
                if show_status_codes {
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

pub fn return_http_client(timeout: u64) -> Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .danger_accept_invalid_certs(true)
        .trust_dns(true)
        .use_native_tls()
        .build()
        .expect("Failed to create HTTP client")
}
