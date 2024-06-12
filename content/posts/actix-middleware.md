+++
title = 'Actix Middleware advanced use case'
date = 2024-06-10T18:04:25+02:00
draft = true
tags = ['rust', 'actix-web', 'middleware', 'web']
description = 'Implement middleware with actix web requires knowledge about rust std modules usually abstracted by the framework. As the documentation, at time of writing, does not provide any detailed examples I decided to write down an article about a real life use case: an authentication middleware. It illustrates practical use of rust std components like async move block, Box::pin, RC smart pointer and explain some implementation details on the actix web framework.'
+++

