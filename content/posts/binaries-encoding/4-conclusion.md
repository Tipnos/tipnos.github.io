+++
title = 'Binaries encoding - 4. Conlusion'
date = 2024-12-21T18:04:25+02:00
tags = ['Avro', 'MessagePack', 'JSON Schema', 'Protobuf', 'event driven', 'event bus', 'binaries encoding']
description = 'Last post of the binary encoding technologies series. It compares the 3 technologies and tries to identify in which situation they should be used.'
[params]
    enableComments = true
+++

# Avro

Avro is a really performant technology with a great feature set. The payload size performance is top notch and its features set is the best to enforce data consistency between services: 
- With the ability to serialize properly complex types
- By forcing each services to deserialize the payload with the schema version it was serialized with
- By helping with schema migration

But it also has major cons:
- The learning curve is steep because of its ecosystem and its intrinsic complexity
- The set up, maintenance and coordination cost between service's team is important when using the single object encoding

Avro suits really well in environment were either consistency between services is a major constraint, events messages size is critic or if events have such a size that the extra bytes of embeded the schema within the payload is negligeable. Last case is when Avro shines because the second major con (set up, maintenance and coordination cost) vanish.

# MessagePack & JSON Schema

To encode messages stored in an event bus, MessagePack combined with JSON schema is a really flexible, easy to adopt and set-up technologies without sacrificing too much performance. It is a good all around choice in environment without high requirements in consistency, serialization or size performance.

The two major cons are:
- As explained precendently, using MessagePack extension types for events messages is not recommanded. Consequently if the system has lots of events with repeated data structures their size can be an issue.
- JSON schema doens't have feature to help schema migration. Thus compared to other technologies, extra coordination between service's team might be required on some schema migrations.

# Protobuf

Protobuf is above all a great protocol for big organizations for which teams synchronization is an issue and therefore must sacrifice consistency in their system. They can leverage the major benefits of having schemas updates being extremely flexible.

Moreover Protobuf is a performant encoding algorithm both in size and serialization speed. Allowing to partially decode messages without schema is a nice feature compared to other binaries encoding technologies relying on schemas. Also the ProtoJSON format is a good selling point in environment that already have JSON implementation. 

When teams synchronization is not an issue, using Protobuf instead of other available technologies is not a straightforward choice and sometimes not recommended. Protobuf has drawbacks that must be taken into consideration:
- The learning curve is steep
- Code generation is by nature cumbersome to work with
- Deserialize protobuf messages is painful for consumers who must add an extra validation and transformation layer above all message types
- The inability to guarantee the presence of fields can be really painful in some situations for both producers and consumers
- The technology is inadapted to environment for which consistency between systems is a major constraint
- Because consumers can perform producers schema updates at their own pace, it can slow down the evolution speed of the system

# Summary table

|                            | Protobuf | MessagePack & JSON schema | Avro |
| -------------------------- | -------- | ------------------------- | ---- |
| Serialization performance  | +++      | ++                        | +    |
| Size performance           | ++       | +                         | +++  |
| Easy to learn              | -        | ++                        | --   |
| Cheap schema update        | +++      | +                         | --   |
| Implementation flexibility | +        | +++                       | -    |
| Consistency                | --       | +                         | +++  |
| Tooling                    | +        | ++                        | -    |
| Ecosystem                  | ++       | ++                        | -    |
| Cheap to set up            | +        | ++                        | -    |