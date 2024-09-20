+++
title = 'Binaries encoding - 4. Conlusion'
date = 2024-09-10T18:04:25+02:00
draft = true
description = 'Last post of the binary encoding technologies series. It compares the 3 technologies and tries to identify in which situation they should be used.'
+++

## Summary table

To have a sense of the benefits in size, let’s compare the size of the hypothetical namespace creation event:

| | Avro | Protobuf | Message pack | JSON |
| --- | --- | --- | --- | --- |
| message size | 203 bytes | 203 bytes | 260 bytes | 368 bytes |