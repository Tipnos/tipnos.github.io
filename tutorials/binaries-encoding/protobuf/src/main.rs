use blog_events::*;
use prost::Message;

mod blog_events {
    include! {concat!(env!("OUT_DIR"), "/_.rs")}
}

fn main() {
    let comment = Comment {
        id: Some(
            "de2df598-9948-4988-b00a-a41c0e287398"
                .parse::<uuid::Uuid>()
                .unwrap()
                .as_bytes()
                .to_vec(),
        ),
        message: Some("I'm sorry, Dave. I'm afraid I can't do that.".to_string()),
        author: Some(Author {
            id: Some(
                "78fc52a3-9f94-43c6-8eb5-9591e80b87e1"
                    .parse::<uuid::Uuid>()
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            ),
            nickname: Some("HAL 9000".to_string()),
        }),
        locale: Some(Locale::EnUs.into()),
        blog_id: Some(
            "b4e05776-fca3-485e-be48-b1758cedd792"
                .parse::<uuid::Uuid>()
                .unwrap()
                .as_bytes()
                .to_vec(),
        ),
        blog_title: Some("Binaries encoding".to_string()),
    };

    let payload = comment.encode_to_vec();

    println!("Nb bytes: {}", payload.len());
    println!("{:02x?}", payload);

    let decoded = Comment::decode(payload.as_slice()).unwrap();

    println!("{:?}", decoded);
}
