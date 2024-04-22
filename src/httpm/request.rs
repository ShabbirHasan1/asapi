// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Response};
use serde_json::Value as JsonValue;
use std::str::FromStr;

use super::methods::HttpMethod;

pub async fn api_request(
    method: HttpMethod,
    url: &str,
    body_params: &[(String, String)],
    headers: &Vec<(String, String)>, // shared: Arc<Mutex<String>>
) -> Result<(String, HeaderMap), reqwest::Error> {
    let client = Client::new();
    let headers_map = get_headers(headers);

    let request_builder = match method {
        HttpMethod::Post => {
            let json_map: serde_json::Map<String, JsonValue> = body_params
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
                .collect();
            let body = JsonValue::Object(json_map);
            println!("{:?}", body);
            client
                .request(method.parse_to_reqwest_method(), url)
                .headers(headers_map)
                .json(&body)
        }
        HttpMethod::Get => client
            .request(method.parse_to_reqwest_method(), url)
            .headers(headers_map),
        HttpMethod::Put => {
            let json_map: serde_json::Map<String, JsonValue> = body_params
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
                .collect();
            let body = JsonValue::Object(json_map);
            println!("{:?}", body);
            client
                .request(method.parse_to_reqwest_method(), url)
                .headers(headers_map)
                .json(&body)
        }
        _ => client
            .request(method.parse_to_reqwest_method(), url)
            .headers(headers_map),
    };

    let response: Response = request_builder.send().await?;
    let status = response.status();
    let response_headers = response.headers().clone();

    match response.text().await {
        Ok(text) => Ok((text, response_headers)),
        Err(_) => Ok((status.to_string(), response_headers)),
    }
}

fn get_headers(vs: &Vec<(String, String)>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let name = HeaderName::from_str(&String::from("Accept")).expect("Invalid header name");
    let value =
        HeaderValue::from_str(&String::from("application/json")).expect("Invalid header name");

    headers.insert(name, value);

    for (key, value) in vs {
        let header_name = HeaderName::from_str(key);
        let header_value = HeaderValue::from_str(value);

        match (header_name, header_value) {
            (Ok(name), Ok(value)) => {
                headers.insert(name, value);
            }
            _ => {
                println!("Error for {key}: {value}");
            }
        };
    }
    println!("{:?}", headers);

    headers
}
