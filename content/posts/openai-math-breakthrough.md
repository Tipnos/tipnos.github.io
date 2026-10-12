+++
title = "What OpenAI's math proofs could mean for programming with AI"
date = 2026-10-11T22:17:31+02:00
tags = ['AI', 'LLM', 'Lean', 'Software Engineering','Mathematics', 'Proof', 'Specification']
description = "On October 6, 2026 OpenAI published more than 700 math manuscripts on GitHub. As a software engineer this raised an obvious paradox for me. If a model can prove theorems that resisted the best mathematicians, why can't it build a complete, complex and reliable application? An application is built from concepts any developer understands. My conclusion is that there is no fundamental reason it can't, as long as the specification is formalized well enough."
[params]
    enableComments = true
+++

# What OpenAI's math proofs could mean for programming with AI

On October 6, 2026 OpenAI [published more than 700 math manuscripts](https://www.nature.com/articles/d41586-026-03196-8) on [GitHub](https://github.com/openai/math). They were produced by an unreleased internal model. Many of them come with proofs formalized in Lean. Lean is a proof assistant whose checker validates every logical step mechanically.

What struck me most was not the number. It was the reaction of the people working in those fields. In [this interview](https://www.youtube.com/watch?v=iRjasLnigsU&t=129s) Hugo Duminil-Copin, Fields medalist in 2022 for his work on probability and statistical physics, explains that OpenAI's model proved the conjectures he was working on. It also proved the ones he planned to work on in the coming years. A research program that would have taken a top mathematician years was done in one batch.

Not everyone celebrated. Part of the math community [strongly criticized the release](https://www.nature.com/articles/d41586-026-03196-8) and [three preprints were later withdrawn](https://aiweekly.co/alerts/openai-posts-722-ai-generated-math-preprints-pulls-three). Still, [these results](https://www.datacamp.com/blog/openai-math-breakthroughs-what-the-latest-results-mean) suggest that AI can now [solve problems](https://openai.com/index/ten-advances-in-mathematics/) that [resisted human researchers for years](https://www.nature.com/articles/d41586-026-01651-0).

As a software engineer this raised an obvious paradox for me. If a model can prove theorems that resisted the best mathematicians, why can't it build a complete, complex and reliable application? An application is built from concepts any developer understands. My conclusion is that there is no fundamental reason it can't, **as long as the specification is formalized well enough**.

The catch is that formalizing it well enough *may be the hardest part*.

## Proving math vs writing code

My first intuition was about training data. A proof always comes with the precise statement it proves. Almost no code comes with a precise description of what it should do. But the more I thought about it, the more I believed the real difference is elsewhere. **In math, a computer can check whether the model's answer is right.**

### Proving math

The math corpus LLMs learn from is unusually clean. It is the result of centuries of work built on shared definitions. Every statement is precise and every proof follows strict logical rules. A theorem means exactly the same thing to every mathematician who reads it.

More importantly, the model's output can be checked perfectly. Because a statement can be written formally, a proof assistant like Lean can verify whether a proof really reaches it. That makes math ideal for [reinforcement learning with verifiable rewards](https://snorkel.ai/rlvr/). The model tries thousands of approaches and only the ones the checker accepts are rewarded. It learns from a clean signal, with no ambiguity and no human opinion in the loop.

### Writing code

Code is the opposite. Almost none of it comes with a specification as precise as in math. When one exists it is usually incomplete, written in loose natural language and out of date. With such a corpus I don't think AI coding assistants can reach the same results as in math. I see three reasons.

1. There is no clean reward signal. Without a precise spec there is nothing to check the code against. Tests help, but they only cover some cases.
2. The model can't learn concepts precisely. A "transaction" or "concurrency-safe" code means slightly different things in the millions of projects it has read. Without one shared definition the model learns a blurry average instead of a precise concept.
3. Users can't write a precise spec in a prompt. There is no standard way to describe what an application should do. A mathematician can state a conjecture in a few unambiguous lines. Anyone describing an application in a prompt leaves gaps that the model has to guess.

Competitive programming is good evidence for this. It is close to math. Each problem is well defined and a precise test suite gives LLMs a clean feedback loop. Guess what? They perform remarkably well. [At the 2025 ICPC World Finals](https://www.techrepublic.com/article/openai-deepmind-icpc-2025-results/) an OpenAI system solved all 12 problems and Google DeepMind's reached gold-medal level. Competitive programming code is not cleaner than other code. **The difference is the checker.**

So maybe the next big step for AI coding is not a smarter model. *It may be a more mathematical way to specify software.* If applications were specified in a proof language like Lean, LLMs might reach a breakthrough similar to the one in math. Major players in the field already seem to be working on it.

## Is Lean the future of software specifications?

### Specifications in Lean

Simple specifications in Lean are readable. Here is what "sorting a list" means.

```lean
theorem sort_correct (l : List Nat) :
  (sort l).Sorted (· ≤ ·) ∧ (sort l).Perm l
```

Any developer can check that it captures what sorting means. The result is ordered and it contains the same elements as the input.

It gets hard very quickly though.

1. The notation. Quantifiers, logic and type theory feel natural to a mathematician but scare most developers.
2. The definitions. To specify a payment system you first need formal definitions of accounts, transactions and what "valid" means.
3. The subtle gaps. Remove the `Perm` part of the spec above and a function that returns an empty list satisfies it. **The proof is valid but the spec is wrong.**

It's worth being precise about what a proof gives you. When Lean accepts a proof, it guarantees that the proposition holds within its formal system. If the proposition says that an implementation satisfies a specification for every possible input, that is exactly what the proof establishes. Tests, by contrast, only check the inputs someone thought of.

But **a proof says nothing about what the spec leaves out**. If the spec is wrong or incomplete, the proof is still valid. It also doesn't automatically cover the code around the verified part, like the user interface or the glue code that calls it. The 3D mesh project mentioned later is a good illustration. Its core is verified, yet a bug in the unverified glue code made some output meshes look broken. And in the end, you still have to trust Lean's implementation and foundations.

Here is what this could look like in a real application. Take a money transfer between two bank accounts. In plain words, the spec could say that the total amount of money doesn't change and that no account goes below zero. The first property could be written like this:

```lean
theorem transfer_keeps_total (a b : Account) (amount : Nat) :
  let (a', b') := transfer a b amount
  a'.balance + b'.balance = a.balance + b.balance
```

The AI would write the transfer code and the proof. A reviewer could focus on a few lines like these to understand one of the guarantees the code provides. This is close to the approach described in the AWS talk mentioned earlier, where the production code stays in Rust and Lean describes what it must do.

But notice what this spec doesn't say. It doesn't guarantee that the requested amount actually changes hands, or address two transfers happening at the same time, a database crash in the middle of a transfer, or fees. Each of these requires additional definitions and guarantees. And that is exactly the hard part.

### Lean in mathematics

Mathematics solved the readability problem with shared libraries. Mathlib is Lean's math library. It defines the basic abstractions such as groups and real numbers. A modern theorem can fit on one line because it relies on thousands of shared definitions that experts have carefully reviewed. The effort behind Mathlib is huge. It started in 2017 and now has [more than 2 million lines of Lean, about 280,000 theorems and over 770 contributors](https://leanprover-community.github.io/mathlib_stats.html). And it is still far from complete.

Software could do the same with its own concepts. But one important piece is missing. Mathematicians had already agreed on every definition Mathlib encodes. **Software has no such agreement to build on.**

### Lean in software

From my research, this is exactly what some major players are trying to build. The idea is to make Lean the common ground and let AI write the proofs. Here is the state of the art as I understand it.

- [CSLib](https://www.sdu.dk/da/forskning/form/news/cslib?sc_lang=en) is an initiative that brings together the University of Southern Denmark, Stanford, UT Austin, Amazon, Google DeepMind and the Lean FRO. The goal is to build the Mathlib of computer science. Its first part formalizes algorithms, data structures and models of computation, with initial work planned for 2026 and 2027. Its second part is [Boole](https://sos-vo.org/node/109677), a verification language built on Lean that reads like pseudocode. It aims to connect Lean to code written in Rust, Python or C++.
- [The Lean FRO](https://lean-lang.org/roadmap) is the organization behind Lean. It wants a standard library like Rust's, but delivered with specifications and proofs. It is also building the tools teams need to verify real software.
- At AWS, a formal verification lead explained in [a 2026 talk](https://www.ai.engineer/talks/lRa9sPaMyy4-your-code-has-bugs-lean4-has-proofs) that they keep production code in Rust while its specification lives in Lean. The talk also described an AI converting the zlib compression library to Lean in about a week.
- Mistral released [Leanstral](https://rits.shanghai.nyu.edu/ai/leanstral-mistrals-open-source-proof-agent-for-lean-4/), an open-source agent that proves AI-generated code meets its Lean specification.

That's real momentum. Still, I think the road to wide adoption across the industry is long.

## Looking ahead

Let me try to look ahead in two steps.

### Agreeing on shared definitions

First, software would need shared definitions, and our industry isn't organized for that. Mathematics is one community with shared journals, peer review and centuries of agreed foundations. Software is a fragmented ecosystem of companies, frameworks and standards bodies. They compete more than they agree. The whole ecosystem would need to change. And most software engineers, myself included, don't have the math background for this work.

There is also an open question at the heart of it. Software is different from math. *Would the abstractions be simple enough to make a specification easier to write, understand and maintain than the code itself?* If not, this is mostly a dead end.

I suspect the answer depends a lot on the kind of software. Algorithms, data structures, compression or geometry are already close to math. They look like natural candidates. Many parts of a real application are much harder to describe formally. Think of user experience, performance, or interactions with messy external systems such as networks and third-party APIs.

### Code as a black box

Now let's imagine it worked. Some recent results point in this direction. [The 3D mesh intersection project](https://github.com/schildep/verified-3d-mesh-intersection) uses 93 lines of specification reviewed by humans. From it, an AI generated over 1,000 lines of implementation and [60,000 lines of proof](https://www.candede.com/news/formal-verification-boosts-ai-generated-code-trust-5c0a1638).

The LLM could then write code as a complete black box. **Humans would read and maintain the spec, not the implementation.** Readable code, maintainable libraries and even programming languages designed for humans would stop mattering. As long as the proof checks against the spec, the AI could write directly in assembly if that were the most efficient. Code would become what compiled binaries are today. Nobody would need to read it.

There are two caveats though. A proof only guarantees what the spec says. Remember the sort function that returns an empty list. In this world, **reviewing specifications becomes the key engineering skill**. And some properties rarely appear in a spec, like performance, memory use, security side channels or the ability to debug in production. Someone, or something, would still need to look at the code for those.

### Signals to watch

I want to be clear that this article shares my thoughts more than it assesses anything. I have real doubts about whether this future will happen. In my opinion these are the signals to watch in the coming years.

1. Do the initiatives above produce software that can be expressed as mathematical concepts, with **specifications much smaller than the code we write today**?
2. Does the ecosystem organize itself so that those abstractions can be written down, agreed upon and maintained over time?

That's a lot of ifs, but the reward could be worth it. We could build software in a way and at a speed we can't even imagine today.

## Sources

- [OpenAI posts 700 maths preprints online: mathematicians are up in arms – Nature](https://www.nature.com/articles/d41586-026-03196-8)
- [AI cracks 80-year-old mathematics challenge — researchers are astonished – Nature](https://www.nature.com/articles/d41586-026-01651-0)
- [Ten advances in mathematics and theoretical computer science – OpenAI](https://openai.com/index/ten-advances-in-mathematics/)
- [OpenAI's AI Math Breakthroughs: What the Latest Results Mean – DataCamp](https://www.datacamp.com/blog/openai-math-breakthroughs-what-the-latest-results-mean)
- [OpenAI posts 722 AI-generated math preprints, pulls three – AI Weekly](https://aiweekly.co/alerts/openai-posts-722-ai-generated-math-preprints-pulls-three)
- [Interview with Hugo Duminil-Copin – YouTube](https://www.youtube.com/watch?v=iRjasLnigsU&t=129s)
- [OpenAI and Google DeepMind Outshine Students at World's Top Coding Contest – TechRepublic](https://www.techrepublic.com/article/openai-deepmind-icpc-2025-results/)
- [Reinforcement Learning With Verifiable Rewards: How Data and Verifiers Shape RLVR – Snorkel AI](https://snorkel.ai/rlvr/)
- [Mathlib statistics – Lean community](https://leanprover-community.github.io/mathlib_stats.html)
- [CSLib announcement – University of Southern Denmark](https://www.sdu.dk/da/forskning/form/news/cslib?sc_lang=en)
- [CSLib: Building a Platform for AI-assisted Formal Verification in Lean – FMCAD 2026](https://sos-vo.org/node/109677)
- [Lean FRO roadmap](https://lean-lang.org/roadmap)
- [Your Code Has Bugs. Lean4 Has Proofs – AI Engineer World's Fair 2026](https://www.ai.engineer/talks/lRa9sPaMyy4-your-code-has-bugs-lean4-has-proofs)
- [Leanstral: Mistral's open-source proof agent for Lean 4](https://rits.shanghai.nyu.edu/ai/leanstral-mistrals-open-source-proof-agent-for-lean-4/)
- [Formal verification boosts AI-generated code trust – Candede](https://www.candede.com/news/formal-verification-boosts-ai-generated-code-trust-5c0a1638)
- [Show HN: Formally verified 3D CSG: Trust 93 lines spec, not 1000 lines AI code – Hacker News](https://news.ycombinator.com/item?id=49083239)
