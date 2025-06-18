+++
title = 'Binaries encoding - 2. JSON'
date = 2024-10-01T18:04:25+02:00
tags = ['JSON', 'Messagepack', 'JSON schema', 'event driven', 'event bus', 'binaries encoding']
description = 'Second post of the binary encoding technologies series. It presents how together JSON, MessagePack and JSON schema can be used to encode messages stored in an event bus.'
[params]
    enableComments = true
+++

Second post of the binary encoding technologies series. It presents how together JSON, MessagePack and JSON schema can be used to encode messages stored in an event bus.

As a reminder each technology is presented according to the following plan:

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
- The ecosystem: quality of documentation and tooling, available resources etc.

# Key differentiation factors

## Encoding algorithm: MessagePack

The key differentiation factor of MessagePack is how it combines a _schemaless_ serialization format with performant serialization speed and size:

- Unlike JSON, it serializes to binary
- Fine grained types to minimize value size as much as possible
- Custom types (aka Extension types) to compensate disadvantages of ineffective serialization (eg: repeated data structures)
- Types are encoded
- Specification doesn't define how to serialize object. The most common implementations are:
  - Array (aka compact form): attributes order is used to match payload values. 
  - Map: attributes name are encoded alongside value (like JSON).

>Note: The compact serialization form and the extension types feature are not suitable for data structure like events consumed by multiple services. Indeed both are fragile to change and introduce additional coupling between services at implementation level which, by the way, break the flexibility of a schemaless serialization format. 

## Schema: JSON Schema

JSON schema is a declarative format for describing structured data in JSON format. Schemas are JSON objects but contrary to avro, they aren't meant to be used during serialization or deserialization. It's mainly used to document shared data structure by defining precise validation rules called [__subschema__](https://json-schema.org/learn/glossary#subschema).

JSON schema embraces the JSON philosophy by having limited types but extended validation rules for each of them. It makes it easy to use for simple cases but without limitation for complex one.

The specification defines in addition to JSON types:
- Two [numeric types](https://json-schema.org/understanding-json-schema/reference/numeric) (instead of one in JSON): integer and number
- [Enum](https://json-schema.org/understanding-json-schema/reference/enum)

Below a bunch of noticeable validation rules by types:

- Object: 
  - Properties defined in a JSON schema are optional by default. A specific validation rule named [required properties](https://json-schema.org/understanding-json-schema/reference/object#required) must be defined for non optional properties.
  - By default a data structure is considered valid if it has properties not defined in its schema. This behavior can be changed by setting the specific keyword [additionalProperties](https://json-schema.org/understanding-json-schema/reference/object#additionalproperties) to false. 

>Note: The two object's default behaviors described above ease schema migration. Indeed thanks to this default behavior, a new version of data which only removed some properties and add new one, will still be valid against an older version of the schema.

- Array: [Contains](https://json-schema.org/understanding-json-schema/reference/array#uniqueItems), [Length](https://json-schema.org/understanding-json-schema/reference/array#uniqueItems), [Uniqueness](https://json-schema.org/understanding-json-schema/reference/array#uniqueItems)
- String: 
  - [Regular expressions](https://json-schema.org/understanding-json-schema/reference/string#regexp)
  - [Built in formats](https://json-schema.org/understanding-json-schema/reference/string#built-in-formats): dates and times, ip adresses, uuid etc.
- Integer and number: [range](https://json-schema.org/understanding-json-schema/reference/numeric#range)

To enable complex validation logic, JSON schema provides keywords to conditionnaly apply combination of validations rules:
- [Composition](https://json-schema.org/understanding-json-schema/reference/combining)
  - allOf: AND
  - anyOf: OR
  - oneOf: XOR 
- [Conditionals](https://json-schema.org/understanding-json-schema/reference/conditionals) allow to define custom conditions

Finally, JSON schema provides keywords to break down schemas into logical units that reference each other as necessary. To do so it exists [schema identifier](https://json-schema.org/understanding-json-schema/structuring#schema-identification) which is a non-relative URI. Schema identifier can be used as [references](https://json-schema.org/understanding-json-schema/structuring#dollarref) in other schemas to avoid duplication. Schemas with references can be bundled to produce a standalone [Coumpond Schema Document](https://json-schema.org/understanding-json-schema/structuring#bundling) usable for validation. It is basically the original one with the referenced schemas appended at the end.

>Note: JSON schema doesn't have any features dedicated to schema versioning or migration (eg: aliases, default values for required properties). It's a major difference compared to protobuf and avro. 

The next section illustrates the key differentiation factors using available tooling.

# Available tooling

## MessagePack

Not so much things to say on MessagePack Serialization and Deserialization. The crate `rmp_serde` is straight forward to use. One thing noticeable, it uses the compact form by default. To use the Map form you have to transform the default Serializer:

```rust 
let payload = Comment::default();

    // Structure map form
    let mut buffer = Vec::new();
    payload
        .serialize(&mut Serializer::with_struct_map(Serializer::new(
            &mut buffer,
        )))
        .unwrap();
```

Deserialization code is the same whatever the type of serialization:
```rust
let payload: Comment = rmp_serde::decode::from_slice(&buffer).unwrap();
```

>Note: JSON can be serialized to MessagePack format but without an optimum size. But the inverse is not true because the tranform function isn't bijective. For example UUID serializes to bytes in MessagePack.

## JSON schema

JSON Schema relies on its ecosystem to provide tooling. Its website maintains an up-to-date list of available tools in a [dedicated page](https://json-schema.org/tools?query=&sortBy=name&sortOrder=ascending&groupBy=toolingTypes&licenses=&languages=&drafts=&toolingTypes=&environments=). As JSON schema is widely adopted there is planty of them in a wide variety of mainstream languages. Below noticeable categories of tools:
- Validators to validates JSON data against schemas
- Bundler to produce Compound Schema Document
- Code to schema
- Schema to code
- Data to schema

For the implementation of the hypothetical blog post comment creation event, the following tools were used:
- A CLI bundler named  [jsonschema](https://github.com/sourcemeta/jsonschema)
- A rust validator named (you guessed it) [jsonschema](https://crates.io/crates/jsonschema)

### Bundler

The JSON schema version of the comment creation event looks like the following:

>JSON schema doesn't defined a file extension, but `.schema.json` is the one widely adopted. It is recognized by some IDEs (eg: VSCode).

locale.schema.json:

```json
{
    "$id": "http://localhost:8080/schemas/locale",  
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "enum": ["en_US", "fr_FR", "ze_CN"]
}
```

author.schema.json:

```json
{
    "$id": "http://localhost:8080/schemas/author",  
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object", 
    "properties": {
        "id": {"type": "string", "format": "uuid"},
        "nickname": {"type": "string"}
    },
    "required": ["id", "nickname"]
}
```

comment.schema.json:

```json
{
    "$id": "http://localhost:8080/schemas/comment",  
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object", 
    "properties": {
        "id": {"type": "string", "format": "uuid"},
        "message": {"type": "string"},
        "locale": {"$ref": "/schemas/locale"},
        "author": {"$ref": "/schemas/author"},
        "blog_id": {"type": "string", "format": "uuid"},
        "blog_title": {"type": "string"}
    },
    "required": ["id", "message", "locale", "blog_id"]
}
```

With the CLI bundler command `jsonschema bundle schemas/comment.schema.json --resolve schemas | > schemas/standalone/comment.schema.json` it generates the Coumpound Schema Document below:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "http://localhost:8080/schemas/comment",
  "type": "object",
  "properties": {
    "author": {
      "$ref": "/schemas/author"
    },
    // ..
  },
  "$defs": {
    "http://localhost:8080/schemas/author": {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "http://localhost:8080/schemas/author",
      // ..
    },
    "http://localhost:8080/schemas/locale": {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "http://localhost:8080/schemas/locale",
      // ..
    }
  }
}
```

> Note: As you can notice, Coumpound Schema Document is exactly the same as the source schema with the _$def_ keywork appended at the end.
>
>CLI can also resolve schemas using http (--http) instead of using a path (--resolve)


### Validator

The jsonschema crate is straight forward to use and can automatically resolve schema references and produce a Coumpound Schema Document. It supports natively resolve schema id URI in HTTP(s) and path references. Custom resolution can be performed by implementing the [`Retrieve`](https://docs.rs/jsonschema/0.26.1/jsonschema/trait.Retrieve.html) trait.

Testing data validation with natively supported references is straightforward:

```rust
let comment_schema: Value = reqwest::blocking::get("http://localhost:8080/schemas/comment")
        .unwrap()
        .json()
        .unwrap();

assert!(jsonschema::validator_for(&comment_schema)
    .unwrap()
    .is_valid(&json));
```

_How should event producers and consumers use JSON schema validators?_

Of course for consumers there is no benefit to validate data at runtime. For producers it's a bit different. Validate data with publicly available schema before publishing it ensures that consumers consume events in the format they expect. It prevents drift bugs between the public schema and the code producing it. Depending on performance requirement, it can be worth the overhead of the additional serialization to JSON and the validation.

Both consumers and producers should use property based testing on their DTOs to enforce they are compliant with publicly available JSON schemas.

>Rust code is available [in github](https://github.com/Tipnos/tipnos.github.io/tree/main/tutorials/binaries-encoding/json)

# Ecosystem

JSON, JSON schema and MessagePack are technologies massively adopted. Thus the ecosystem is rich with tooling available in all mainstrem languages. Every tools I used are well maintained, documented and straight forward to use. 

The JSON schema documentation is top notch. Its format should be familiar to most developers because already used to work with similar format like OpenAPI. The format is easy to adopt and use because of how it is designed. Indeed its default behaviors are appropriated for shared data among multiple services. Only strongly typing object properties is a good start. Plus the ability to use HTTP URI as schema identifier and having tools supporting schema resolution out of the box makes it also easy to set up. 

Finally because the schema is not required for serialization, with the wide variety of tooling types available, each team can work as they are used to: code gen, bundling etc.

[Next post]({{< ref "3-protobuf" >}}) presents how Protobuf can be used to encode messages stored in an event bus. 