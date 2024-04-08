// // Use example: load_api_spec("ruta/al/archivo.json")
// pub fn load_api_spec(json_route: &str) -> Result<(), IOError>{
//     println!("{}", json_route);
//     let data = fs::read_to_string(json_route)?; //.expect("No se pudo leer el archivo");
//     println!("{data}");
//     let p: OpenApi = serde_json::from_str(&data).unwrap();

//     for (path, item) in p.paths {
//         if let Some(operation) = item.get {
//             println!("Ruta GET: {}", path);
//             print_parameters(operation.parameters);
//         }

//         if let Some(operation) = item.post {
//             println!("Ruta POST: {}", path);
//             print_parameters(operation.parameters);

//             if let Some(request_body) = operation.requestBody {
//                 println!("Body: {:?}", request_body.content);
//             }
//         }

//         if let Some(operation) = item.delete {
//             println!("Ruta DELETE: {}", path);
//             print_parameters(operation.parameters);
//         }

//         if let Some(operation) = item.put {
//             println!("Ruta PUT: {}", path);
//             print_parameters(operation.parameters);
//             if let Some(request_body) = operation.requestBody {
//                 println!("Body: {:?}", request_body.content);
//             }
//         }
//     }
//     Ok(())
// }

// fn print_parameters(parameters: Option<Vec<Parameter>>) {
//     if let Some(parameters) = parameters {
//         for parameter in parameters {
//             match parameter.in_.as_str() {
//                 "header" => println!(
//                     "Parámetro del header: {} - Tipo: {} - Requerido: {} - Descripción: {}",
//                     parameter.name, parameter.type_, parameter.required, parameter.description.unwrap_or_default()
//                 ),
//                 "path" => println!(
//                     "Parámetro del path: {} - Tipo: {} - Requerido: {} - Descripción: {}",
//                     parameter.name, parameter.type_, parameter.required, parameter.description.unwrap_or_default()
//                 ),
//                 _ => {}
//             }
//         }
//     }
// }
