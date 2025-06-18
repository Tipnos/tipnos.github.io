+++
title = 'Binaries encoding - 1. Avro'
date = 2024-09-07T18:04:25+02:00
tags = ['Avro', 'event driven', 'event bus', 'binaries encoding']
description = 'First post of a series that compares different binary encoding technologies to encode messages stored in an event bus. It presents how the Avro protocol can be used for this specific use case.'
[params]
    enableComments = true
+++

First post of a series that compares different binary encoding technologies to encode messages stored in an event bus. After some research the technologies usable in a polyglot production environment are:

- [Avro](https://avro.apache.org/)
- JSON with [Message Pack](https://msgpack.org/) and [JSON schema](https://json-schema.org/).
- [Protobuf](https://protobuf.dev/)

The first 3 posts presents for each technology:

- The key differentiation factors 
- The available tooling by showing the implementation of an hypothetical blog post comment creation event:

```json
{
    "id": "de2df598-9948-4988-b00a-a41c0e287398",
    "message": "I’m sorry, Dave. I’m afraid I can’t do that.",
    "locale": "en_US",
    "author": {
        "id": "78fc52a3-9f94-43c6-8eb5-9591e80b87e1",
        "nickname": "HAL 9000",
    },
    "blog_id": "b4e05776-fca3-485e-be48-b1758cedd792",
    "blog_title": "Binaries encoding"
}
```
>The event content is what we can expect in an Event-Carried State Transfer pattern
- The ecosystem: quality of documentation and tooling, available resources etc.

The last post compares the 3 technologies and tries to identify in which situation they should be used.

Let's begin with Avro as it's the least popular and thus should require more focus. The presentation below is quite detailed because of the documentation not being straightforward for this use case. 

# Key differentiation factors

## Encoding algorithm

The key differentiation factor of Avro is its minimalist encoding algorithm:

- Types are not encoded
- Field names or indexes are not encoded
- Field’s values are encoded in the order of the schema 

__Consequently: Consumers cannot decode or partially read without the exact same version of the schema used by the producer to encode it.__ 

>Note: Avro specification defines [resolution algorithm](https://avro.apache.org/docs/1.12.0/specification/#schema-resolution) when the consumer's service is programmed to decode with an older version of the schema. To perform the resolution both version of the schema are required!

Avro specification defines two different protocols for consumers to recover the producer's schema:

1. For large files with millions of records (eg: Hadoop), the schema is encoded within the payload.
2. For database records stored one-by-one (eg: event bus), the schema is too much overhead. Avro’s specification defines a [single object encoding](https://avro.apache.org/docs/++version++/specification/#single-object-encoding) algorithm composed of the fingerprint of the object’s schema. 

__Thus for this use case, a schema registry is required as consumers need to retrieve it from the parsed fingerprint__. 

Basically the single object encoding algorithm is (copied from documentation):
- A two-byte marker, `C3 01`, to show that the message is Avro and uses this single-record format (version 1).
- The 8-byte little-endian `CRC-64-AVRO` fingerprint of the object’s schema.
- The Avro object encoded using Avro’s binary encoding.

>Note: for the decoding process, Avro uses attribute `name` in schema to match against code base structure (or classes) attribute name.

## Schema

An Avro schema is a valid JSON object that defines __one and only one__ custom type, a [`Record`](https://avro.apache.org/docs/++version++/specification/#complex-types), Enum etc. A Record is the equivalent of a JSON object or a message in protobuf. Below a bunch of noticable features:

- Fields are required by default. Optional field can be declared using the enum or union type (eg: `{null, long}`).
- For schema evolution purposes:
  - A default value can be optionally provided, only used when reading instances that lack the field.
  - Aliases can be optionally provided as alternate names.
  - The union type allows to have fields with multiple types.
- To avoid name collision, namespaces can be defined.
- Potentially useful logical types (date, UUID, timestamp, duration).
- Record definition can be nested in a parent record.

Next paragraph describes the available tooling through an example.

# Available tooling

## Schema management

Avro provides an [IDL](https://avro.apache.org/docs/++version++/idl-language/) (Interface description languages) to ease schema management. It's less verbose than JSON and above all, allows to use types defined in other schemas. To generate a standalone JSON schema from IDL, the [IDL tool](https://dlcdn.apache.org/avro/) duplicates the definitions of extern types and nests them. These features are fundamental when maintaining a complex schema by factorizing types definition. 

The comment creation event below written in Avro IDL illustrates the process.

locale.avdl:

```avro
enum Locale {
    en_US,
    fr_FR,
    zh_CN
}
```

comment.avdl:

```avro
import idl "locale.avdl";


record Author {
    uuid id;
    string nickname;
}

record Comment {
    uuid id;
    string message;
    Locale locale;
    Author author; 
    uuid blog_id;
    string blog_title;
}
```

With the command `idl2schemata` one standalone schema `.avsc` file is created for each custom type. Below the content of `Comment.avsc`:

```json
{
  "type" : "record",
  "name" : "Comment",
  "fields" : [ {
    "name" : "id",
    "type" : {
      "type" : "string",
      "logicalType" : "uuid"
    }
  }, {
    "name" : "message",
    "type" : "string"
  }, {
    "name" : "locale",
    "type" : {
      "type" : "enum",
      "name" : "Locale",
      "symbols" : [ "en_US", "fr_FR", "zh_CN" ]
    }
  }, {
    "name" : "author",
    "type" : {
      "type" : "record",
      "name" : "Author",
      "fields" : [ {
        "name" : "id",
        "type" : {
          "type" : "string",
          "logicalType" : "uuid"
        }
      }, {
        "name" : "nickname",
        "type" : "string"
      } ]
    }
  }, {
    "name" : "blog_id",
    "type" : {
      "type" : "string",
      "logicalType" : "uuid"
    }
  }, {
    "name" : "blog_title",
    "type" : "string"
  } ]
}
```

> Note: As you can notice, to have a standalone schema, records are copied and nested.

Avro IDL provides more features (eg: annotations, import JSON schema, protocol for RPC). But for this use case, it's pretty all we've got for schema management. Nothing is provided for the schema registry, an [issue](https://issues.apache.org/jira/browse/AVRO-1124) about implementing one is marked as "Won't do" by the maintainers.

A minimal schema registry should have following features:
- Expose an `HashMap<fingerprint, schema>` for consumers. 
- Push to the HashMap for producers.

Then each team could maintain privately their schema in IDL format. To release a new version, they push standalone schema generated by the IDL tool as described above.

> Note: The IDL tool is only avaible as a .jar

Let's have a look at how to use the `Namespace` Avro schema in rust using the official crate [apache-avro](https://crates.io/crates/apache-avro).

## Rust implementation

Good news! The encoding/decoding implementation is compatible with `serde`! Structure can be described as we're used to with an implementation of the `AvroSchema` trait:

```rust
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
```

> Note: Macros could be used to assert schema validity at compile time.

Then encoding a message is straightforward:

```rust
let payload = Comment {
    id: "de2df598-9948-4988-b00a-a41c0e287398".parse().unwrap(),
    message: "I’m sorry, Dave. I’m afraid I can’t do that.".to_string(),
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

println!(
    "Comment encoded with Single object encoding algorithm: {:02x?}",
    buffer
); // c3 01 (Avro two byte marker) | fe c8 1c 7e df 23 cc dc (schema fingerprint) | 48 64 65 32 64 ... (payload)
```

To decode a message it is possible to use the local schema version or make a resolution by fetching the schema from the registry.

With local schema (not usable in production):
```rust 
let comment = SpecificSingleObjectReader::<Comment>::new()
    .unwrap()
    .read(&mut buffer.as_slice())
    .unwrap();
```

With schema resolution: 
```rust 
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
```

>Rust code is available [in github](https://github.com/Tipnos/tipnos.github.io/tree/main/tutorials/binaries-encoding/avro)

# Ecosystem

I had 0 knowledge of avro before writing this article, below is my feedback on its ecosystem. 

First, the documentation is not use case orientated but descriptive. Second there is almost no examples/tutorials on the internet. Finally the crate documentation, outside of the README, isn't straightforward. However the feature set looks like complete according to the specification. But the API is not that userfriendly, the rust implementation is a bit surprizing sometimes.

To put everything together as you read it above required many: documentation search -> code read -> test cycles. 

Everything is versionned in [a mono repo](https://github.com/apache/avro) at time of writing. The specification is implemented in [mainstream programming languages](https://github.com/apache/avro/tree/main/lang).

IMO the available resources about Avro are too limited for an easy adoption in a company. Moreover the tooling is "old fashioned" and of lower quality than mainstream technologies. Some efforts should be made on adoption by:

1. Explaining the Avro protocol and its usage for this usecase.
2. Providing detailed examples with guidelines in all programming languages.

[Next post]({{< ref "2-json" >}}) presents how together JSON, MessagePack and JSON schema can be used to encode messages stored in an event bus. 