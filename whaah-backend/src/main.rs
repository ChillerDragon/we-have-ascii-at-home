use actix_web::{get, web, App, HttpServer, Responder, post};
use actix_web::HttpRequest;
use actix_cors::Cors;
use chrono::{Local, DateTime};
use std::fs;
use sqlite;
use sqlite::State;
use serde::Deserialize;

#[get("/")]
async fn index() -> impl Responder {
    println!("index hit\n");
    return "{\"error\": \"invalid endpoint\"}\n";
}

#[get("/casts")]
async fn casts() -> impl Responder {
    println!("GET /casts");
    let mut paths: Vec<String> = Vec::new();
    for entry in fs::read_dir("../frontend/casts").unwrap() {
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

#[get("/get_views/{name}")]
async fn get_views(cast: web::Path<String>, req: HttpRequest) -> impl Responder {
    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    while let Ok(State::Row) = stmt.next() {
        let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
        let ip_addr = req.peer_addr().unwrap().to_string();
        // let now: DateTime<Local> = Local::now();
        // let ts: String = now.to_string();
        println!("Requesting view for cast_id={} ip={} user_agent={} cast={}", cast_id, &ip_addr, &user_agent, &cast);
        // TODO: bind this properly before mr bobby tables sees this
        //       also this whole code is such a hack
        let query = format!("select count(*) as views from views where CastID = {};", cast_id);
        let mut views = 0;
        connection
            .iterate(query, |pairs| {
                for &(_name, value) in pairs.iter() {
                    views = value.unwrap().parse().unwrap();
                    return true;
                }
                true
            })
            .unwrap();
        return format!("{{\"views\": {}}}", &views);

        // let query = "select count(*) as views from views where CastID = ?;";
        // let mut stmt = connection.prepare(query).unwrap();
        // while let Ok(State::Row) = stmt.next() {
        //     let views: i64 = stmt.read::<i64, _>("views").unwrap();
        //     return format!("{{\"views\": {}}}", &views);
        // }
    }
    println!("[get_views] Failed to get id for cast={}\n", cast);
    return format!("{{\"error\": \"failed to get views for {}\"}}", &cast);
}

// Horrible api endpoints i know...
// This should be a post request or something like that
// But I want to later add some dirty hacks to track views
// Even from browsers without javascript
// by loading this url with a link or image tag
#[get("/view/{name}")]
async fn view(cast: web::Path<String>, req: HttpRequest) -> impl Responder {
    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    while let Ok(State::Row) = stmt.next() {
        let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
        let ip_addr = req.peer_addr().unwrap().to_string();
        let now: DateTime<Local> = Local::now();
        let ts: String = now.to_string();
        println!("Adding view for cast_id={} ip={} user_agent={} cast={}", cast_id, &ip_addr, &user_agent, &cast);
        let query = concat!(
            "INSERT INTO views ",
            "(CastID, IP, Timestamp, UserAgent, Tracker, Ref) ",
            "VALUES ",
            "(?     , ? , ?        , ?        , ?      , ?)");
        let mut stmt = connection.prepare(query).unwrap();
        stmt.bind((1, cast_id)).unwrap();
        stmt.bind((2, ip_addr.as_str())).unwrap();
        stmt.bind((3, ts.as_str())).unwrap();
        stmt.bind((4, user_agent)).unwrap();
        stmt.bind((5, "todo tracker")).unwrap();
        stmt.bind((6, "todo ref")).unwrap();
        while let Ok(State::Row) = stmt.next() {
            println!("Failed to insert view\n");
            return format!("{{\"error\": \"failed to add view for {}\"}}", &cast);
        }
        println!("Inserted view\n");
        return format!("{{\"message\": \"add view for {}\"}}", &cast);
    }

    println!("Tried to view invalid cast: {}", &cast);
    return format!("{{\"error\": \"failed to add view for {}\"}}", &cast);
}

#[get("/comments/{name}")]
async fn get_comments(cast: web::Path<String>) -> impl Responder {
    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    while let Ok(State::Row) = stmt.next() {
        // let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        // let query = "SELECT * FROM comments WHERE CastID = ?";
        println!("Gettings comments for cast: {}", &cast);
        return format!("{{\"comments\": [{{\"message\": \"add view for {}\"}}}}", &cast);
    }

    println!("Tried to get comments for invalid cast: {}", &cast);
    return format!("{{\"error\": \"failed to get comments for {}\"}}", &cast);
}


#[derive(Deserialize)]
struct Comment {
        author: String,
        message: String,
}

#[post("/comments/{cast}")]
async fn post_comment(cast: web::Path<String>, comment: web::Json<Comment>, req: HttpRequest) -> impl Responder {
    println!("got comment={}\n", comment.message);
    println!("{:#?}\n", req);
    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    while let Ok(State::Row) = stmt.next() {
        let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
        let ip_addr = req.peer_addr().unwrap().to_string();
        let now: DateTime<Local> = Local::now();
        let ts: String = now.to_string();
        println!("Adding comment for cast_id={} ip={} user_agent={} author={} message='{}' cast={}", cast_id, &ip_addr, &user_agent, &comment.author, &comment.message, &cast);
        let query = concat!(
            "INSERT INTO commentsa ",
            "(CastID, Author, Message, IP, Timestamp, UserAgent, Tracker, Ref) ",
            "VALUES ",
            "(?     , ?     , ?      ,? , ?        , ?        , ?      , ?)");
        let mut stmt = connection.prepare(query).unwrap();
        stmt.bind((1, cast_id)).unwrap();
        stmt.bind((2, comment.author.as_str())).unwrap();
        stmt.bind((3, comment.message.as_str())).unwrap();
        stmt.bind((4, ip_addr.as_str())).unwrap();
        stmt.bind((5, ts.as_str())).unwrap();
        stmt.bind((6, user_agent)).unwrap();
        stmt.bind((7, "todo tracker")).unwrap();
        stmt.bind((8, "todo ref")).unwrap();
        let res = stmt.next();
        println!("Inserted comment stmt: {:#?}\n", res);
        return format!("{{\"message\": \"add comment for {}\"}}", &cast);
    }
    println!("Tried to comment invalid cast: {}", &cast);
    return format!("{{\"error\": \"failed to add view for {}\"}}", &cast);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting backend at http://127.0.0.1:8180 ...\n");
    HttpServer::new(||
                    App::new()
                    .service(index)
                    .service(view)
                    .service(get_views)
                    .service(casts)
                    .service(post_comment)
                    .service(get_comments)
                    .wrap(Cors::permissive())
		)
        .bind(("127.0.0.1", 8180))?
        .run()
        .await
}
