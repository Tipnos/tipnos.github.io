+++
title = 'Authentication actix middleware'
date = 2024-06-10T18:04:25+02:00
draft = true
tags = ['rust', 'actix-web', 'middleware', 'web']
description = 'Implement middleware with actix web requires knowledge about rust std modules usually abstracted by the framework. As the documentation, at time of writing, does not provide any detailed examples I decided to write a tutorial on an authentication middleware. It illustrates use case of rust std components like async move block, Box::pin, RC smart pointer and explains some implementation details on the actix web framework.'
+++

Implement middleware with [actix web](https://actix.rs/) requires knowledge about rust std modules usually abstracted by the framework. As the documentation, at time of writing, does not provide any detailed examples I decided to write a tutorial on an authentication middleware. It illustrates use case of rust std components like async move block, Box::pin, RC smart pointer and explains some implementation details on the actix web framework.

All code examples below can be found in my github public [blog repository]().

# Dependencies

```toml
[dependencies]
    actix-web = "4.7.0"
```

