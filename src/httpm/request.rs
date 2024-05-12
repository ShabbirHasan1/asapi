// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{multipart, Body, Client, Response};
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::info;

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
            info!("{:?}", body);
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
                info!("Error for {key}: {value}");
            }
        };
    }
    info!("{:?}", headers);

    headers
}

#[derive(Debug)]
pub enum UploadError {
    IOError(String),
    RequestError(String),
    MultipartError(String),
}

pub async fn upload_file(
    file_path: &Path,
    file_name: String,
    url: &str,
    body_params: &[(String, String)],
    headers: &Vec<(String, String)>, // shared: Arc<Mutex<String>>
) -> Result<String, UploadError> {
    let client = Client::new();
    let file = File::open(file_path).await.map_err(|err| UploadError::IOError(err.to_string()))?;
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_body = Body::wrap_stream(stream);

    let form = multipart::Part::stream(file_body)
        .file_name(file_name)
        .mime_str(mime_type.essence_str())
        .map_err(|err| UploadError::MultipartError(err.to_string()))
        .map(|part| {
            let mut form = multipart::Form::new().part("file", part);
            for (k, v) in body_params {
                form = form.text(k.clone(), v.clone());
            }
            form
        });

    match form {
        Ok(form) => {
            let result = client
                .post(url)
                .headers(get_headers(headers))
                .multipart(form)
                .send()
                .await
                .map_err(|e| UploadError::RequestError(e.to_string()))?
                .text()
                .await
                .map_err(|e| UploadError::RequestError(e.to_string()))?;
            Ok(result)
        }
        Err(err) => Err(err)
    }
}

// async fn upload_files() -> Result<(), Error> {
//     let client = Client::new();
//     let urls = vec![
//         "/path/to/your/first_file.txt",
//         "/path/to/your/second_file.pdf",
//         // Añade más archivos según sea necesario
//     ];
//     let url = "http://example.com/upload";

//     let mut form = reqwest::multipart::Form::new();

//     // Añade cada archivo al formulario
//     for file_path in urls {
//         let path = Path::new(&file_path);
//         let file_name = path.file_name().unwrap().to_str().unwrap();

//         let mut file = File::open(&path).await?;
//         let mut contents = Vec::new();
//         file.read_to_end(&mut contents).await?;

//         // Asume que el nombre del campo es el mismo que el nombre del archivo, ajusta según sea necesario
//         form = form.part(file_name, reqwest::multipart::Part::bytes(contents).file_name(file_name).mime_type(from_path(&path).first_or_octet_stream()));
//     }

//     let response = client.post(url)
//         .multipart(form)
//         .send()
//         .await?;

//     if response.status().is_success() {
//         println!("Archivos cargados exitosamente");
//     } else {
//         println!("Error al cargar archivos: {}", response.status());
//     }

//     Ok(())
// }
