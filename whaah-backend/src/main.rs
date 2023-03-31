use actix_web::{get, web, App, HttpServer, Responder};
use std::fs;

#[get("/")]
async fn index() -> impl Responder {
    println!("index hit\n");
    return "{\"error\": \"invalid endpoint\"}\n";
}

#[get("/casts")]
async fn casts() -> impl Responder {
    println!("GET /casts");
    let mut paths: Vec<String> = Vec::new();
    for entry in fs::read_dir("../casts").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        // let file_path = path.to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        paths.push(String::from(file_name));
    }
    return json::stringify(paths);
}

#[get("/a/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting backend at http://127.0.0.1:8080 ...\n");
    HttpServer::new(|| App::new()
                    .service(index)
                    .service(hello)
                    .service(casts))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
