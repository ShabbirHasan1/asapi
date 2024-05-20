// -------------------------------------------------------------------------
// Copyright (C) 2023 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{multipart, Body, Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::info;

use crate::httpm::methods::HttpMethod;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Request {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub multipart: bool,
    pub body_params: Vec<(String, String, bool)>, // key -- value -- has files
    pub headers_params: Vec<(String, String)>,
}

pub async fn api_request(
    method: HttpMethod,
    url: &str,
    body_params: &[(String, String, bool)],
    headers: &Vec<(String, String)>, // shared: Arc<Mutex<String>>
) -> Result<(String, HeaderMap), reqwest::Error> {
    let client = Client::new();
    let headers_map = get_headers(headers);

    let request_builder = match method {
        HttpMethod::Post => {
            let json_map: serde_json::Map<String, JsonValue> = body_params
                .iter()
                .map(|(k, v, _)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
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
                .map(|(k, v, _)| (k.clone(), serde_json::from_str(v).unwrap_or_default()))
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

use std::time::Duration;

pub async fn upload_file(
    file_path: &PathBuf,
    url: &str,
    body_params: &[(String, String, bool)],
    headers: &Vec<(String, String)>, // shared: Arc<Mutex<String>>
) -> Result<String, UploadError> {
    // let client = Client::new();
    let client = ClientBuilder::new()
        .pool_idle_timeout(Some(Duration::from_secs(20))) // Por defecto es 90, lo reduzco.
        .build()
        .unwrap();
    let file = File::open(file_path)
        .await
        .map_err(|err| UploadError::IOError(err.to_string()))?;
    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_body = Body::wrap_stream(stream);

    let file_name = file_path
        .file_name()
        .and_then(OsStr::to_str)
        .map(String::from);
    if let Some(name) = file_name {
        let form = multipart::Part::stream(file_body)
            .file_name(name)
            .mime_str(mime_type.essence_str())
            .map_err(|err| UploadError::MultipartError(err.to_string()))
            .map(|part| {
                let mut form = multipart::Form::new().part("file", part);
                for (k, v, _) in body_params {
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
            Err(err) => Err(err),
        }
    } else {
        Err(UploadError::IOError("File no tiene nombre".to_string()))
    }
}

pub async fn upload_files_in_body_params(
    url: &str,
    headers: &Vec<(String, String)>,
    body_params: &[(String, String, bool)],
    // has_files: &[bool],
    files: &[Vec<PathBuf>],
) -> Result<String, UploadError> {
    let client = ClientBuilder::new()
        .pool_idle_timeout(Some(Duration::from_secs(20)))
        .build()
        .unwrap();
    let mut form = multipart::Form::new();

    for (idx, (k, v, param_with_files)) in body_params.iter().enumerate() {
        // let (k, v, f) = &body_params[idx];
        let files = &files[idx];

        if *param_with_files {
            // Si el parámetro tiene un archivo, lo metemos como `part` en el `form`.
            let file_path = &files[0];
            if files.len() == 1 {
                let file_name = file_path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(String::from);
                if file_name.is_none() {
                    return Err(UploadError::IOError(String::from(
                        "El archivo no tiene nombre. Es un directorio?",
                    )));
                }

                let file = File::open(file_path)
                    .await
                    .map_err(|err| UploadError::IOError(err.to_string()))?;
                let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();
                let stream = FramedRead::new(file, BytesCodec::new());
                let file_body = Body::wrap_stream(stream);

                let part = multipart::Part::stream(file_body)
                    .file_name(file_name.unwrap())
                    .mime_str(mime_type.essence_str())
                    .map_err(|err| UploadError::MultipartError(err.to_string()))?;
                form = form.part(k.clone(), part);
            } else if files.len() > 1 {
                // Si tenemos maś de un archivo, los metemos en el mismo campo del part,
                // que será el nombre del campo.
                for file_path in files {
                    let file_name = file_path
                        .file_name()
                        .and_then(OsStr::to_str)
                        .map(String::from);

                    if file_name.is_none() {
                        return Err(UploadError::IOError(String::from(
                            "El archivo no tiene nombre. Es un directorio?",
                        )));
                    }
                    println!("Uploaing files");
                    let file = File::open(file_path)
                        .await
                        .map_err(|err| UploadError::IOError(err.to_string()))?;
                    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();
                    let stream = FramedRead::new(file, BytesCodec::new());
                    let file_body = Body::wrap_stream(stream);

                    let part = multipart::Part::stream(file_body)
                        .file_name(file_name.unwrap())
                        .mime_str(mime_type.essence_str())
                        .map_err(|err| UploadError::MultipartError(err.to_string()))?;
                    form = form.part("files", part);
                }
            }
        } else {
            // Si no tenemos archivos, metemos el parámetro como texto.
            form = form.text(k.clone(), v.clone());
        }
    }

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

/// Para saber cómo se hace esto de muchos archivos en el mismo `part`
/// esta conversación de stackoverflow es muy clarificadora.
/// https://stackoverflow.com/questions/36674161/http-multipart-form-data-multiple-files-in-one-input
pub async fn upload_files(
    file_paths: Vec<PathBuf>,
    url: &str,
    body_params: &[(String, String, bool)],
    headers: &Vec<(String, String)>,
) -> Result<String, UploadError> {
    let client = ClientBuilder::new()
        .pool_idle_timeout(Some(Duration::from_secs(20)))
        .build()
        .unwrap();
    let mut form = multipart::Form::new();
    // let mut parts: Vec<multipart::Part> = Vec::with_capacity(file_paths.len());
    for file_path in &file_paths {
        let file_name = file_path
            .file_name()
            .and_then(OsStr::to_str)
            .map(String::from);

        if file_name.is_none() {
            return Err(UploadError::IOError(String::from(
                "El archivo no tiene nombre. Es un directorio?",
            )));
        }
        println!("Uploaing files");
        let file = File::open(file_path)
            .await
            .map_err(|err| UploadError::IOError(err.to_string()))?;
        let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);

        let part = multipart::Part::stream(file_body)
            .file_name(file_name.unwrap())
            .mime_str(mime_type.essence_str())
            .map_err(|err| UploadError::MultipartError(err.to_string()))?;
        form = form.part("files", part);
    }

    for (k, v, _) in body_params {
        form = form.text(k.clone(), v.clone());
    }

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
