// // Example:
// // let openapi = export_workspace_to_openapi(&workspace);
// // let json = serde_json::to_string_pretty(&openapi).expect("Failed to serialize to JSON");
// fn export_workspace_to_openapi(workspace: &Workspace) -> OpenApi {
//     let mut paths = HashMap::new();

//     for request in &workspace.requests {
//         let path_item = paths.entry(request.url.clone()).or_insert_with(PathItem::default);

//         // let operation = Operation {
//         //     summary: request.name.clone(),
//         //     responses: HashMap::new(), // Aquí tendrías que llenar los códigos de respuesta y sus descripciones
//         //     // ... otros campos
//         //     parameters: "casa",
//         //     requestBody: request.body_params.clone()
//         // };

//         // match request.method {
//         //     HttpMethod::Get => path_item.get = Some(operation),
//         //     HttpMethod::Post => path_item.post = Some(operation),
//         //     HttpMethod::Put => path_item.put = Some(operation),
//         //     HttpMethod::Delete => path_item.delete = Some(operation),
//         //     // ... otros métodos HTTP
//         // }
//     }

//     OpenApi {
//         openapi: "3.0.0".to_string(),
//         info: Info {
//             title: workspace.name.clone(),
//             version: "1.0.0".to_string(),
//         },
//         paths,
//     }
// }
