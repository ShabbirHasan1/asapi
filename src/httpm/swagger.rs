// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::Deserialize;
use serde_json::Result;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::prelude::*;

use super::methods::HttpMethod;
use super::request::Request;

#[derive(Deserialize)]
pub struct Swagger {
    pub host: String,
    pub paths: BTreeMap<String, PathItem>,
}

#[derive(Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
}

#[derive(Deserialize)]
pub struct Operation {
    pub summary: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_field: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub type_: Option<String>,
}

impl Display for Swagger {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Swagger Paths:\n")?;
        for (path, item) in &self.paths {
            write!(f, "{}\n{}\n", path, item)?;
        }
        Ok(())
    }
}

impl Display for PathItem {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut result = String::new();
        if let Some(get) = &self.get {
            result.push_str(&format!("GET: {}\n", get));
        }
        if let Some(post) = &self.post {
            result.push_str(&format!("POST: {}\n", post));
        }
        if let Some(put) = &self.put {
            result.push_str(&format!("PUT: {}\n", put));
        }
        if let Some(delete) = &self.delete {
            result.push_str(&format!("DELETE: {}\n", delete));
        }
        write!(f, "{}", result)
    }
}

// impl Display for Operation {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(
//             f,
//             "{} - {}",
//             self.operation_id.as_ref().unwrap_or(&"No ID".to_string()),
//             self.summary.as_ref().unwrap_or(&"No Summary".to_string())
//         )
//     }
// }
impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str(&format!(
            "Operation ID: {}, Summary: {}\n",
            self.operation_id.as_ref().unwrap_or(&"No ID".to_string()),
            self.summary.as_ref().unwrap_or(&"No Summary".to_string())
        ));

        if let Some(params) = &self.parameters {
            for param in params {
                if param.in_field == "header" {
                    result.push_str(&format!(
                        "Header - {}: {}\n",
                        param.name,
                        param.type_.as_ref().unwrap_or(&"unknown type".to_string())
                    ));
                } else if param.in_field == "body" {
                    result.push_str(&format!(
                        "Body Param - {}: {}\n",
                        param.name,
                        param.type_.as_ref().unwrap_or(&"unknown type".to_string())
                    ));
                }
            }
        }
        write!(f, "{}", result)
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) - {} - Required: {}",
            self.name,
            self.in_field,
            self.type_.as_ref().unwrap_or(&"No Type".to_string()),
            self.required.unwrap_or(false)
        )
    }
}

// #[derive(Debug)]
// pub struct Request {
//     pub name: String,
//     pub method: HttpMethod,
//     pub url: String,
//     pub multipart: bool,
//     pub body_params: Vec<(String, String)>,
//     pub headers_params: Vec<(String, String)>,
// }

pub fn load_file(file_path: &str) -> Result<Swagger> {
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");
    let swagger: Swagger = serde_json::from_str(&contents)?;
    Ok(swagger)
}

pub fn load_file_opt(file_path: &str) -> Option<Swagger> {
    load_file(file_path).ok()
}

pub fn create_requests(s: &Swagger) -> Vec<Request> {
    let mut requests = Vec::new();
    for (path, item) in &s.paths {
        if let Some(operation) = &item.get {
            let request = Request::from_operation(&s.host, HttpMethod::Get, path, operation);
            requests.push(request);
        }
        if let Some(operation) = &item.post {
            let request = Request::from_operation(&s.host, HttpMethod::Post, path, operation);
            requests.push(request);
        }
        if let Some(operation) = &item.put {
            let request = Request::from_operation(&s.host, HttpMethod::Put, path, operation);
            requests.push(request);
        }
        if let Some(operation) = &item.delete {
            let request = Request::from_operation(&s.host, HttpMethod::Delete, path, operation);
            requests.push(request);
        }
    }
    requests
}

impl Request {
    fn from_operation(host: &str, method: HttpMethod, path: &str, operation: &Operation) -> Self {
        let body_params = operation
            .parameters
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|p| p.in_field == "body")
            .map(|p| (p.name.clone(), p.type_.clone().unwrap_or_default(), false))
            .collect();

        let headers_params = operation
            .parameters
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|p| p.in_field == "header")
            .map(|p| (p.name.clone(), p.type_.clone().unwrap_or_default()))
            .collect();

        let url = format!("https://{}{}", host, path);
        Request {
            name: url.clone(), // operation.operation_id.clone().unwrap_or_default(),
            method,
            url,
            multipart: false, // Simplification for the example
            body_params,
            headers_params,
        }
    }
}
