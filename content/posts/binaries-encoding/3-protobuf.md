+++
title = 'Binaries encoding - 3. Protobuf'
date = 2024-11-07T18:04:25+02:00
tags = ['Protobuf', 'event driven', 'event bus', 'binaries encoding']
description = 'Third post of the binary encoding technologies series. It presents how Protobuf can be used to encode messages stored in an event bus.'
[params]
    enableComments = true
+++

Third post of the binary encoding technologies series. It presents how Protobuf can be used to encode messages stored in an event bus.

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

>Note: This article is about the last revision of Protobuf at time of writing, proto 3.

# Key differentiation factors

## Encoding algorithm

The key differentiation factor of Protobuf is its [Tag-Length-Value](https://en.wikipedia.org/wiki/Type%E2%80%93length%E2%80%93value) (aka TVL) efficient encoding algorithm with specific behaviors for forward- and backward-compatibility across changes to messages definitions.

A Protobuf message is a series of key-value pairs with keys being a number between 1 and 536,870,911 and the values one of the 6 defined [wire types](https://protobuf.dev/programming-guides/encoding/#structure). TVL's tag is the combination of the key number and value's wire type. 

To serialize data structures of supported programming languages into its TVL format, Protobuf also defines `.proto types` which have a:

- one to one relation with programming language types
- many to one relation with wire types
 
It means that several programming language types are encoded the same way (ie serialized to the same wire type).

>Note: The protobuf spec defines two correlation tables for both relations: [programming language types](https://protobuf.dev/programming-guides/proto3/#scalar) and [wire types](https://protobuf.dev/programming-guides/encoding/#structure).

Consequently to serialize a data structure, producers must assign for each attribute a unique number, called field number, which acts as an identifier. As during serialization attributes type information is lost (many to one relation), consumers must be aware of both the attribute field number and the proto type in order to properly deserialize messages.

To be forward and backwards compatible, the Protobuf encoding algorithm does not provide any guarantee that any attribute will be present in a binary message. Consequently, consumers must manage their absence. Protobuf specification defines two [Field presence](https://protobuf.dev/programming-guides/field_presence/) behaviors that producers can choose from for each attribute:
- Implicit presence: default value are not serialized. If not present it deserializes to the default value.  
>Note: With this field presence behavior, consumers cannot make the difference between an unset value and a default value.
- Explicit presence: explicitly set values are always serialized, even if it is the default value. Unset attributes are not serialized. Unset attributes are deserialized to the equivalent of `null` in the programming language.

>Note: Protobuf [specification defines](https://protobuf.dev/programming-guides/field_presence/#presence-in-proto3-apis) which proto types can have an implicit or explicit field presence. Specification also recommends to use explicit presence as much as possible.

Since the Protobuf binary format is a stream of tagged, self-delimiting values, by definition, it contains no information about unset values. Therefore, producers must provide to consumers field presence behavior for each attribute.

The consequences of the Protobuf enconding algorithm is that consumers need a schema from producers to be able to deserialize their messages. To adress this issue the Protobuf specification defines a `.proto schema` described in the next section.

>Note: Protobuf encoded messages can be partially decoded without schema for inspection. Partially means that initial proto types, field names and unset values can't be deducted.

>Note: The Protobuf specification also defines a [ProtoJSON format](https://protobuf.dev/programming-guides/json/) to share data with systems that do not support standard protobuf.

## Schema

A Proto schema allows to define two custom types [Enumeration](https://protobuf.dev/programming-guides/proto3/#enum) and [Message](https://protobuf.dev/programming-guides/proto3/#simple). 

A message defines for each of attribute its:
- Proto type
- Unique field number 
- Field presence by (un)set the `optional` keyword

An Enumeration defines its predefined list of values with their field number. They also must have one default value. It must be the first element and have its field number set to 0.

>Note: The default value is mandatory to enable implicit presence for enumeration.

Additionaly .proto schema provides convenient tooling:
- Package to prevent name clashes between custom types name 
- Import other schemas definition to allow breaking down schemas into logical units that reference each other
- Reserved field numbers list to make sure producers do not reuse field numbers of deprecated fields.
- Message type definition features: [Any](https://protobuf.dev/programming-guides/proto3/#any), [Oneof](https://protobuf.dev/programming-guides/proto3/#oneof), [Nested types](https://protobuf.dev/programming-guides/proto3/#nested)

>Note: Because protobuf encoded message doesn't store its schema version, reusing field numbers can have [severe consequences](https://protobuf.dev/programming-guides/proto3/#consequences) for backward and forward compatibility. 

# Available tooling

Because of the Protobuf encoding algorithm intrisic complexity, developers can easily make mistakes while implementing serialization and deserialization from .proto schemas. Protobuf adresses this issue by relying on codegen for both producers and consumers. The Protobuf team maintains the [protoc CLI](https://github.com/protocolbuffers/protobuf/releases) to generate code in all supported programming languages. At time of writing, the Rust codegen is in experimental state. The best alternative is the [prost crate](https://crates.io/crates/prost).

No schema registry is provided. Most of projects version schemas in a dedicated git respository so other team can import them as git submodule.

## Example .proto schemas

The Protobuf schema version of the comment creation event looks like the following:

locale.proto:

```proto
syntax = "proto3";

enum Locale {
    en_US = 0;
    fr_FR = 1;
    zh_CN = 2;
}
```

author.proto:

```proto
syntax = "proto3";

message Author {
    optional bytes id = 1;
    optional string nickname = 2;
}
```

comment.proto:

```proto
syntax = "proto3";

import "locale.proto";
import "author.proto";

message Comment {
    optional bytes id = 1;
    optional string message = 2;
    Author author = 3; // implicit presence not allowed, optional keyword not required
    optional Locale locale = 4;
    optional bytes blog_id = 5;
    optional string blog_title = 6;
}
```

>Note: There is optional keyword everywhere to follow the [protobuf recommandation](https://protobuf.dev/programming-guides/field_presence/#background) of having as much as possible explicit presence

## Example generated code

Locale:
```rust 
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Locale {
    EnUs = 0,
    FrFr = 1,
    ZhCn = 2,
}
```

Author:
```rust
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Author {
    #[prost(bytes = "vec", optional, tag = "1")]
    pub id: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "2")]
    pub nickname: ::core::option::Option<::prost::alloc::string::String>,
}
```

Comment:
```rust
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Comment {
    #[prost(bytes = "vec", optional, tag = "1")]
    pub id: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "2")]
    pub message: ::core::option::Option<::prost::alloc::string::String>,
    /// implicit presence not allowed, optional keyword not required
    #[prost(message, optional, tag = "3")]
    pub author: ::core::option::Option<Author>,
    #[prost(enumeration = "Locale", optional, tag = "4")]
    pub locale: ::core::option::Option<i32>,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub blog_id: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "6")]
    pub blog_title: ::core::option::Option<::prost::alloc::string::String>,
}
```

>Note: Prost explains in their [FAQ](https://github.com/tokio-rs/prost?tab=readme-ov-file#faq) why it is not possible to use serde.

The important thing to notice is that every struct attributes are `Option`. It is one of the major constraint of the Protobuf encoding algorithm: consumers can never assume the presence of a field. Consequently, it means that consumers must handle cases which from a business logic perspective doesn't make sense (eg: the author attribute being `None`). Moreover they usually cannot perform business logic on generated code, but must first transform them to DTOs compliant with their business logic.

The major benefit is that schemas updates are extremely flexible. Producers can safely update them without any synchronization with consumers. Thus consumers are at anytime backward and forward compatible. They can catch updates at their own pace.

>Note: With implicit presence, scalar attributes (all of them except author) wouldn't be `Option`. But their default values would be equivalent of `Option::None` in explicit presence. Constraint would be the same but performing business logic on struct would be far less convenient and idiomatic.

>Rust code is available [in github](https://github.com/Tipnos/tipnos.github.io/tree/main/tutorials/binaries-encoding/protobuf)

# Ecosystem

The ecosystem is pretty simple as the protobuf team maintains everything themselves: documentation, tooling and libraries. Both are implemented for all mainstream programming languages except minor exception like rust being WIP at time of writing. 

The official documentation is complete but require some efforts to understand the relatively complex protocol with all its counterintuitive, sometimes surprizing constraints and side effects.

Finally Protobuf is a technology massively adopted, there is plenty of resources available to learn from.

[Next and last post]({{< ref "4-conclusion" >}}) compares the 3 technologies and tries to identify in which situation they should be used. 