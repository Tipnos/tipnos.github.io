use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Comment {
    id: Uuid,
    message: String,
    locale: Locale,
    author: Author,
    blog_id: Uuid,
    blog_title: String,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            id: "de2df598-9948-4988-b00a-a41c0e287398".parse().unwrap(),
            message: "I'm sorry, Dave. I'm afraid I can't do that.".to_string(),
            locale: Locale::EnUs,
            author: Author {
                id: "78fc52a3-9f94-43c6-8eb5-9591e80b87e1".parse().unwrap(),
                nickname: "HAL 9000".to_string(),
            },
            blog_id: "b4e05776-fca3-485e-be48-b1758cedd792".parse().unwrap(),
            blog_title: "Binaries encoding".to_string(),
        }
    }
}

impl Comment {
    #[cfg(test)]
    fn get_bundle_schema() -> Value {
        include_str!("../schemas/standalone/comment.schema.json")
            .parse()
            .unwrap()
    }

    fn get_schema() -> Value {
        include_str!("../schemas/comment.schema.json")
            .parse()
            .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    id: Uuid,
    nickname: String,
}

impl Author {
    fn get_schema() -> Value {
        include_str!("../schemas/author.schema.json")
            .parse::<Value>()
            .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Locale {
    #[serde(rename = "en_US")]
    EnUs,
    #[serde(rename = "fr_FR")]
    FrFr,
    #[serde(rename = "zh_CN")]
    ZhCN,
}

impl Locale {
    fn get_schema() -> Value {
        include_str!("../schemas/locale.schema.json")
            .parse::<Value>()
            .unwrap()
    }
}

async fn publish_message() -> impl Responder {
    let payload = Comment::default();

    // Structure map form
    let mut buffer = Vec::new();
    payload
        .serialize(&mut Serializer::with_struct_map(Serializer::new(
            &mut buffer,
        )))
        .unwrap();

    println!("Payload size: {} bytes", buffer.len());
    println!("Payload: {:02x?}", buffer);

    let payload: Comment = rmp_serde::decode::from_slice(&buffer).unwrap();
    println!("Comment decoded from structure map form: {:?}", payload);

    // Compact form, structure serialize as array
    let buffer = rmp_serde::encode::to_vec(&payload).unwrap();

    println!("Compact form payload size: {} bytes", buffer.len());
    println!("Compact form payload: {:02x?}", buffer);

    let payload: Comment = rmp_serde::decode::from_slice(&buffer).unwrap();
    println!("Comment decoded from compact form: {:?}", payload);

    HttpResponse::Ok().json(payload)
}

async fn get_comment_schema() -> impl Responder {
    HttpResponse::Ok().json(Comment::get_schema())
}

async fn get_author_schema() -> impl Responder {
    HttpResponse::Ok().json(Author::get_schema())
}

async fn get_locale_schema() -> impl Responder {
    HttpResponse::Ok().json(Locale::get_schema())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/schemas/comment", web::get().to(get_comment_schema))
            .route("/schemas/author", web::get().to(get_author_schema))
            .route("/schemas/locale", web::get().to(get_locale_schema))
            .route("/publish-message", web::get().to(publish_message))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// start server first to pass test
#[test]
fn schema_validation() {
    let json = serde_json::json!(Comment::default());

    // Validate with coumpound Schema Document
    assert!(jsonschema::validator_for(&Comment::get_bundle_schema())
        .unwrap()
        .is_valid(&json));

    // Validate with schema -> automatically resolve schema references
    let comment_schema: Value = reqwest::blocking::get("http://localhost:8080/schemas/comment")
        .unwrap()
        .json()
        .unwrap();

    assert!(jsonschema::validator_for(&comment_schema)
        .unwrap()
        .is_valid(&json));
}
