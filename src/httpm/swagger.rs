// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Swagger {
    pub paths: HashMap<String, PathItem>,
}

#[derive(Debug, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
}

#[derive(Debug, Deserialize)]
pub struct Operation {
    pub summary: Option<String>,
    pub operationId: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Debug, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_field: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub type_: Option<String>,
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug)]
pub struct Request {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub multipart: bool,
    pub body_params: Vec<(String, String)>,
    pub headers_params: Vec<(String, String)>,
}

pub fn load_swagger_file(file_path: &str) -> Result<Swagger> {
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");
    let swagger: Swagger = serde_json::from_str(&contents)?;
    Ok(swagger)
}

fn create_requests(swagger: &Swagger) -> Vec<Request> {
    let mut requests = Vec::new();
    for (path, item) in &swagger.paths {
        if let Some(operation) = &item.get {
            let request = Request::from_operation(HttpMethod::Get, path, operation);
            requests.push(request);
        }
        if let Some(operation) = &item.post {
            let request = Request::from_operation(HttpMethod::Post, path, operation);
            requests.push(request);
        }
        if let Some(operation) = &item.put {
            let request = Request::from_operation(HttpMethod::Put, path, operation);
            requests.push(request);
        }
        if let Some(operation) = &item.delete {
            let request = Request::from_operation(HttpMethod::Delete, path, operation);
            requests.push(request);
        }
    }
    requests
}

impl Request {
    pub fn from_operation(method: HttpMethod, url: &str, operation: &Operation) -> Self {
        let body_params = operation
            .parameters
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|p| p.in_field == "body")
            .map(|p| (p.name.clone(), p.type_.clone().unwrap_or_default()))
            .collect();

        let headers_params = operation
            .parameters
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter(|p| p.in_field == "header")
            .map(|p| (p.name.clone(), p.type_.clone().unwrap_or_default()))
            .collect();

        Request {
            name: operation.operationId.clone().unwrap_or_default(),
            method,
            url: url.to_string(),
            multipart: false, // Simplification for the example
            body_params,
            headers_params,
        }
    }
}
