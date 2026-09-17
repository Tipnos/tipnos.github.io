+++
title = 'Binary encoding - 4. Conclusion'
date = 2024-12-21T18:04:25+02:00
tags = ['Avro', 'MessagePack', 'JSON Schema', 'Protobuf', 'event driven', 'event bus', 'binary encoding']
description = 'Last post of the binary encoding technologies series. It compares the three technologies and tries to identify in which situations they should be used.'
[params]
    enableComments = true
+++

## Avro

Avro is a really performant technology with a great feature set. The payload size performance is top notch and its feature set is the best for enforcing data consistency between services: 
- With the ability to serialize properly complex types
- By forcing each service to deserialize the payload with the schema version it was serialized with
- By helping with schema migration

But it also has major cons:
- The learning curve is steep because of its ecosystem and its intrinsic complexity
- The setup, maintenance and coordination cost between services' teams is important when using single object encoding

Avro suits really well in environments where either consistency between services is a major constraint, event message size is critical, or events have such a size that the extra bytes of embedding the schema within the payload is negligible. The last case is when Avro shines because the second major con (setup, maintenance and coordination cost) vanishes.

## MessagePack & JSON Schema

To encode messages stored in an event bus, MessagePack combined with JSON schema is a really flexible, easy-to-adopt-and-set-up technology without sacrificing too much performance. It is a good all around choice in environments without high requirements in consistency, serialization or size performance.

The two major cons are:
- As explained previously, using MessagePack extension types for event messages is not recommended. Consequently, if the system has lots of events with repeated data structures, their size can be an issue.
- JSON schema doesn't have features to help with schema migration. Thus compared to other technologies, extra coordination between services' teams might be required on some schema migrations.

## Protobuf

Protobuf is above all a great protocol for big organizations for which team synchronization is an issue and therefore must sacrifice consistency in their system. They can leverage the major benefits of having schema updates being extremely flexible.

Moreover, Protobuf is a performant encoding algorithm, both in size and serialization speed. Being able to partially decode messages without a schema is a nice feature compared to other binary encoding technologies relying on schemas. Also, the ProtoJSON format is a good selling point in environments that already have a JSON implementation. 

When team synchronization is not an issue, using Protobuf instead of other available technologies is not a straightforward choice and sometimes not recommended. Protobuf has drawbacks that must be taken into consideration:
- The learning curve is steep
- Code generation is by nature cumbersome to work with
- Deserializing protobuf messages is painful for consumers, who must add an extra validation and transformation layer across all message types
- The inability to guarantee the presence of fields can be really painful in some situations for both producers and consumers
- The technology is ill-suited to environments for which consistency between systems is a major constraint
- Because consumers can catch up with producers' schema updates at their own pace, it can slow down the evolution speed of the system

## Summary table

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
