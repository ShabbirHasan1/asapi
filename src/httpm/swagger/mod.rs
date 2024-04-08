// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

// pub mod parser;
// pub mod serializer;

// #[derive(Debug, Serialize, Deserialize)]
// struct OpenApi {
//     openapi: String,
//     info: Info,
//     paths: HashMap<String, PathItem>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Info {
//     title: String,
//     description: String,
//     termsOfService: String,
//     version: String,
// }

// #[derive(Default, Debug, Serialize, Deserialize)]
// struct PathItem {
//     get: Option<Operation>,
//     post: Option<Operation>,
//     delete: Option<Operation>,
//     put: Option<Operation>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Operation {
//     summary: String,
//     description: String,
//     operationId: String,
//     parameters: Option<Vec<Parameter>>,
//     requestBody: Option<RequestBody>,
//     responses: HashMap<String, Response>,
// }

// #[derive(Debug, Deserialize, Serialize, Default)]
// struct Response {
//     description: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Parameter {
//     name: String,
//     #[serde(rename = "in")]
//     in_: String,
//     description: Option<String>,
//     required: bool,
//     exploded: bool,
//     schema: Schema,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct RequestBody {
//     description: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct MediaType {
//     schema: Schema,
// }

// #[derive(Debug, Default, Serialize, Deserialize)]
// struct Schema {
//     #[serde(rename = "type")]
//     type_: String,
//     default: String,
// }
