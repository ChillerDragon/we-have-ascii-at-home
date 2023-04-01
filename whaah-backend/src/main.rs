use actix_cors::Cors;
use actix_web::HttpRequest;
use actix_web::{get, post, web, App, HttpServer, Responder};
use chrono::{DateTime, Local};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlite;
use sqlite::State;
use std::fs;

/// Comments are ratelimited per ip
/// one ip across all casts can only comment
/// every `COMMENT_DELAY_IN_SECONDS` seconds
const COMMENT_DELAY_IN_SECONDS: i64 = 60;

/// Views are ratelimited per ip
/// meaning if someone views the same
/// cast with the same ip there have to be
/// more than `VIEW_COUNT_DELAY_IN_MINUTES` minutes
/// passed or the view will not be counted
const VIEW_COUNT_DELAY_IN_MINUTES: i64 = 10;

#[derive(Serialize, Deserialize)]
struct Comment {
    author: String,
    message: String,
    timestamp: String,
}

#[derive(Serialize)]
struct ErrorMsg {
    error: String,
}

#[derive(Serialize)]
struct InfoMsg {
    message: String,
}

#[derive(Serialize)]
struct GetViews {
    views: i64,
}

fn info_msg(msg: String) -> String {
    let info = InfoMsg {
        message: msg.clone(),
    };
    return serde_json::to_string(&info).unwrap();
}

fn err_msg(msg: String) -> String {
    let err = ErrorMsg { error: msg.clone() };
    return serde_json::to_string(&err).unwrap();
}

fn get_ip(req: &HttpRequest) -> String {
    // let ip_addr = req.peer_addr().unwrap().to_string(); // with port
    let ip_addr = req.peer_addr().unwrap().ip().to_string(); // w out port
    if ip_addr == "127.0.0.1" || ip_addr == "::1" {
        // Prefer custom header setup by proxy
        // https://github.com/ddnet/ddnet/tree/42982759a93c4d0f74a8c2f07fe94a65b789bafb/src/mastersrv#reverse-proxy
        let forward = req.headers().get("Connecting-IP");
        if let Some(forward) = forward {
            let ips: Vec<&str> = forward.to_str().unwrap().split(",").collect();
            return ips[0].to_string();
        }
        // if client is local host
        // check if there is a proxy in front of the api backend
        let forward = req.headers().get("x-forwarded-for");
        if let Some(forward) = forward {
            let ips: Vec<&str> = forward.to_str().unwrap().split(",").collect();
            return ips[0].to_string();
        }
    }
    return ip_addr;
}

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
        let ip_addr = get_ip(&req);
        println!(
            "Requesting view for cast_id={} ip={} user_agent={} cast={}",
            cast_id, &ip_addr, &user_agent, &cast
        );
        // TODO: bind this properly before mr bobby tables sees this
        //       also this whole code is such a hack
        let query = format!(
            "select count(*) as views from views where CastID = {};",
            cast_id
        );
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

        let views_msg = GetViews { views: views };
        return serde_json::to_string(&views_msg).unwrap();

        // let query = "select count(*) as views from views where CastID = ?;";
        // let mut stmt = connection.prepare(query).unwrap();
        // while let Ok(State::Row) = stmt.next() {
        //     let views: i64 = stmt.read::<i64, _>("views").unwrap();
        //     return format!("{{\"views\": {}}}", &views);
        // }
    }
    println!("[get_views] Failed to get id for cast={}\n", cast);
    return err_msg(format!("failed to get views for {}", &cast));
}

// Horrible api endpoints i know...
// This should be a post request or something like that
// But I want to later add some dirty hacks to track views
// Even from browsers without javascript
// by loading this url with a link or image tag
#[get("/view/{cast}")]
async fn view(cast: web::Path<String>, req: HttpRequest) -> impl Responder {
    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    while let Ok(State::Row) = stmt.next() {
        let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
        let ip_addr = get_ip(&req);
        let now: DateTime<Local> = Local::now();
        let now = now.with_timezone(now.offset());
        let ts: String = now.to_rfc3339();
        println!(
            "Adding view for cast_id={} ip={} user_agent={} cast={}",
            cast_id, &ip_addr, &user_agent, &cast
        );
        let query = concat!(
            "SELECT Timestamp ",
            "FROM views ",
            "WHERE IP = ? ",
            "ORDER BY Timestamp ",
            "DESC LIMIT 1"
        );
        let mut stmt = connection.prepare(query).unwrap();
        stmt.bind((1, ip_addr.as_str())).unwrap();

        while let Ok(State::Row) = stmt.next() {
            let last_ts = stmt.read::<String, _>("Timestamp").unwrap();
            let last_date = DateTime::parse_from_rfc3339(&last_ts).unwrap();
            let last_view_mins_ago: i64 = (now - last_date).num_minutes();
            println!(
                "  last view from this ip was {} ({} minutes ago)\n",
                last_ts, last_view_mins_ago
            );
            if last_view_mins_ago < VIEW_COUNT_DELAY_IN_MINUTES {
                let err = ErrorMsg {
                    error: format!(
                        "Not counting another view since your last was {}/{} minutes ago",
                        last_view_mins_ago,
                        VIEW_COUNT_DELAY_IN_MINUTES
                    ),
                };
                return serde_json::to_string(&err).unwrap();
            }
        }

        let query = concat!(
            "INSERT INTO views ",
            "(CastID, IP, Timestamp, UserAgent, Tracker, Ref) ",
            "VALUES ",
            "(?     , ? , ?        , ?        , ?      , ?)"
        );
        let mut stmt = connection.prepare(query).unwrap();
        stmt.bind((1, cast_id)).unwrap();
        stmt.bind((2, ip_addr.as_str())).unwrap();
        stmt.bind((3, ts.as_str())).unwrap();
        stmt.bind((4, user_agent)).unwrap();
        stmt.bind((5, "todo tracker")).unwrap();
        stmt.bind((6, "todo ref")).unwrap();
        while let Ok(State::Row) = stmt.next() {
            println!("  Failed to insert view\n");
            return err_msg(format!("failed to add view for {}", &cast));
        }
        println!("  Inserted view\n");
        return info_msg(format!("add view for {}", &cast));
    }

    println!("  Tried to view invalid cast: {}", &cast);
    return err_msg(format!("failed to add view for {}", &cast));
}

#[get("/comments/{name}")]
async fn get_comments(cast: web::Path<String>) -> impl Responder {
    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    let mut comments: Vec<Comment> = Vec::new();

    while let Ok(State::Row) = stmt.next() {
        let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        // TODO: do not hardcode a limit of 100
        //       use pagination instead
        let query = "SELECT * FROM comments WHERE CastID = ? ORDER BY Timestamp DESC LIMIT 100";
        let mut stmt = connection.prepare(query).unwrap();
        stmt.bind((1, cast_id)).unwrap();
        println!("Gettings comments for cast: {}", &cast);

        while let Ok(State::Row) = stmt.next() {
            let author: String = stmt.read::<String, _>("Author").unwrap();
            let message: String = stmt.read::<String, _>("Message").unwrap();
            let timestamp: String = stmt.read::<String, _>("Timestamp").unwrap();
            let cmt = Comment {
                author: author,
                message: message,
                timestamp: timestamp,
            };
            comments.push(cmt);
        }

        return serde_json::to_string(&comments).unwrap();
    }

    println!("Tried to get comments for invalid cast: {}", &cast);
    return err_msg(format!("failed to get comments for {}", &cast));
}

#[post("/comments/{cast}")]
async fn post_comment(
    cast: web::Path<String>,
    comment: web::Json<Comment>,
    req: HttpRequest,
) -> impl Responder {
    println!("got comment={}\n", comment.message);
    println!("{:#?}\n", req);

    let re_author = Regex::new(r"^[a-zA-Z0-9_-]{1,32}$").unwrap();
    let re_comment = Regex::new(r"^[a-zA-Z0-9\.,:!?=*#\\()\[\]{}_\n -]{1,2048}$").unwrap();
    if !re_comment.is_match(&comment.message) {
        let err = ErrorMsg {
            error: format!("comment message did not match {}", re_comment),
        };
        return serde_json::to_string(&err).unwrap();
    }
    if !re_author.is_match(&comment.author) {
        let err = ErrorMsg {
            error: format!("comment author did not match {}", re_author),
        };
        return serde_json::to_string(&err).unwrap();
    }

    let connection = sqlite::open("../db/whaah.db").unwrap();
    let query = "SELECT ID FROM casts WHERE Filename = ?";

    let mut stmt = connection.prepare(query).unwrap();
    stmt.bind((1, cast.to_string().as_str())).unwrap();

    while let Ok(State::Row) = stmt.next() {
        let cast_id: i64 = stmt.read::<i64, _>("ID").unwrap();
        let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
        let ip_addr = get_ip(&req);
        let now: DateTime<Local> = Local::now();
        let now = now.with_timezone(now.offset());
        let ts: String = now.to_rfc3339();
        println!(
            "Adding comment for cast_id={} ip={} user_agent={} author={} message='{}' cast={}",
            cast_id, &ip_addr, &user_agent, &comment.author, &comment.message, &cast
        );



        let query = concat!(
            "SELECT Timestamp ",
            "FROM comments ",
            "WHERE IP = ? ",
            "ORDER BY Timestamp ",
            "DESC LIMIT 1"
        );
        let mut stmt = connection.prepare(query).unwrap();
        stmt.bind((1, ip_addr.as_str())).unwrap();

        while let Ok(State::Row) = stmt.next() {
            let last_ts = stmt.read::<String, _>("Timestamp").unwrap();
            let last_date = DateTime::parse_from_rfc3339(&last_ts).unwrap();
            let last_comment_secs_ago: i64 = (now - last_date).num_seconds();
            println!(
                "  last comment from this ip was {} ({} seconds ago)\n",
                last_ts, last_comment_secs_ago
            );
            if last_comment_secs_ago < COMMENT_DELAY_IN_SECONDS {
                let secs_left = COMMENT_DELAY_IN_SECONDS - last_comment_secs_ago;
                let err = ErrorMsg {
                    error: format!(
                        "Please wait {} seconds before writing another comment",
                        secs_left
                    ),
                };
                return serde_json::to_string(&err).unwrap();
            }
        }


        let query = concat!(
            "INSERT INTO comments ",
            "(CastID, Author, Message, IP, Timestamp, UserAgent, Tracker, Ref) ",
            "VALUES ",
            "(?     , ?     , ?      ,? , ?        , ?        , ?      , ?)"
        );
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
        return info_msg(format!("add comment for {}", &cast));
    }
    println!("Tried to comment invalid cast: {}", &cast);
    return err_msg(format!("failed to add view for {}", &cast));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting backend at http://127.0.0.1:8180 ...\n");
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(view)
            .service(get_views)
            .service(casts)
            .service(post_comment)
            .service(get_comments)
            .wrap(Cors::permissive())
    })
    .bind(("127.0.0.1", 8180))?
    .run()
    .await
}
