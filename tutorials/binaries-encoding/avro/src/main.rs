use apache_avro::{
    from_avro_datum, from_value, AvroSchema, Schema, SpecificSingleObjectReader,
    SpecificSingleObjectWriter,
};
use serde::{Deserialize, Serialize};
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

impl AvroSchema for Comment {
    fn get_schema() -> Schema {
        Schema::parse_str(include_str!("../schemas/standalone/Comment.avsc"))
            .expect("Invalid Comment Avro schema")
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    id: Uuid,
    nickname: String,
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

async fn fetch_schema(_fingerprint: &[u8]) -> Schema {
    // should fetch the schema instead
    Comment::get_schema()
}

#[tokio::main]
async fn main() {
    let payload = Comment {
        id: "de2df598-9948-4988-b00a-a41c0e287398".parse().unwrap(),
        message: "I'm sorry, Dave. I'm afraid I can't do that.".to_string(),
        locale: Locale::EnUs,
        author: Author {
            id: "78fc52a3-9f94-43c6-8eb5-9591e80b87e1".parse().unwrap(),
            nickname: "HAL 9000".to_string(),
        },
        blog_id: "b4e05776-fca3-485e-be48-b1758cedd792".parse().unwrap(),
        blog_title: "Binaries encoding".to_string(),
    };
    let mut buffer = Vec::new();
    SpecificSingleObjectWriter::<Comment>::with_capacity(10)
        .unwrap()
        .write_ref(&payload, &mut buffer)
        .unwrap();

    println!("Payload size: {}", buffer.len());
    println!(
        "Comment encoded with Single object encoding algorithm: {:02x?}",
        buffer
    ); // c3 01 (Avro two byte marker) | fe c8 1c 7e df 23 cc dc (schema fingerprint) | 48 64 65 32 64 ... (payload)

    // decode with local schema only
    let comment = SpecificSingleObjectReader::<Comment>::new()
        .unwrap()
        .read(&mut buffer.as_slice())
        .unwrap();

    println!("Comment decoded with local schema: {:?}", comment);

    // decode with schema resolution
    buffer.drain(0..2); // remove avro two-byte marker
    let fingerprint: Vec<_> = buffer.drain(0..8).collect(); // extract schema fingerprint

    let comment: Comment = from_value(
        &from_avro_datum(
            &fetch_schema(fingerprint.as_slice()).await, // fetch corresponding schema from the registry or local cache
            &mut buffer.as_slice(),
            Some(&Comment::get_schema()),
        )
        .unwrap(),
    )
    .unwrap();

    println!("Comment decoded with schema resolution: {:?}", comment);
}
