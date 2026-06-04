# Announcement copy

Draft copy per venue, in three tiers (short / medium / long). Posting is manual —
paste the tier that matches the venue from the table in [`PLAN.md`](PLAN.md), with
a looping clip or stills attached.

**Before posting, this file must pass [`honesty-checklist.md`](honesty-checklist.md).**
Any first-person line is kept only if it is true for the person posting; cut it
otherwise. Fill the `[link]` placeholders:
- project page — https://kleer001.github.io/galapagos3/
- repo — https://github.com/kleer001/galapagos3
- releases — https://github.com/kleer001/galapagos3/releases

---

## SHORT — Mastodon / Bluesky (attach a looping clip)

> Galápagos: a GPU evolutionary art generator in Rust. Every image is a formula —
> three mathematical expression trees per pixel, run through an evolvable color
> model. Roll 16, pick the ones you like, breed the next generation. Prebuilt
> binaries for Linux / macOS / Windows.
>
> [project page]
>
> #generative #proceduralart #rustlang

---

## SHORT — r/generative / r/proceduralgeneration (clip is the post; this is the comment)

> Built in Rust on wgpu compute shaders. Each image is three expression trees
> producing three channels per pixel, interpreted through a per-individual color
> model (HSV / RGB / HSL / CMY / YUV) — and the color model is itself an evolvable
> trait. You breed a grid of 16 by selecting the ones you like; crossover and
> mutation make the next generation. The clip is the companion desktop widget,
> which animates one saved genome on a seamless loop. Prebuilt binaries and an
> interactive explainer at the link. [project page]

---

## MEDIUM — Hacker News (Show HN) / Reddit text

**Title (HN):** `Show HN: Galápagos – evolve math expression trees into 2K art on the GPU`

> Galápagos is an interactive evolutionary art generator, inspired by Karl Sims'
> 1991 work on evolving expressions into images. It's written in Rust and runs the
> evaluation on the GPU through wgpu compute shaders.
>
> Every image is a formula. Three independent expression trees produce three
> channels per pixel, which are interpreted through a per-individual color model
> (HSV, RGB, HSL, CMY, or YUV) to become RGB. The color model is an evolvable
> trait, so a lineage can drift between color spaces as it breeds. You're shown a
> grid of 16, you pick the ones you like, and the system breeds the next
> generation with subtree crossover and mutation across 47 operators.
>
> Under the hood, each genome compiles from a typed expression tree to
> 1024-instruction stack-machine bytecode, and all 16 tiles render in a single GPU
> compute dispatch. A second binary is an always-on-top desktop widget that brings
> a saved genome to life — it nudges the genome's numeric values within a bounded
> neighborhood of the seed while leaving the expression structure fixed, so every
> frame is a real, sharp genome rather than a cross-dissolve. It has a seamless
> loop mode for recording.
>
> Prebuilt binaries for Linux, macOS (Intel and Apple Silicon), and Windows are on
> the Releases page, plus one-line bootstrap scripts that install Rust and build
> if you'd rather. There's an interactive explainer that walks through tree growth,
> pixel evaluation, coloring, and evolution with live animations.
>
> Project page and explainer: [project page]
> Source: [repo]

---

## MEDIUM — r/rust (technical angle)

**Title:** `Galápagos – a GPU evolutionary art generator in Rust (wgpu compute, a bytecode VM on the GPU)`

> Galápagos breeds mathematical expression trees into images by human selection —
> the genetic-programming art idea from Karl Sims (1991), built on a modern Rust +
> wgpu stack.
>
> The parts that might interest r/rust:
>
> - **Typed genome → flat bytecode.** Genomes are strongly-typed expression trees
>   (Scalar / Vec2 / Color node types) that compile to a fixed 1024-instruction
>   stack-machine program before GPU upload. The typing prevents invalid genomes
>   at construction; the flat bytecode is what the GPU stack machine runs.
> - **One dispatch for the whole population.** All 16 tiles render in a single
>   wgpu compute dispatch, indexed into a flattened population buffer — no
>   per-genome GPU call overhead.
> - **WGSL constants generated from `config.rs` at build time**, so the shader and
>   the Rust side can't drift out of sync.
> - eframe 0.34 for the UI, wgpu 29 (Vulkan / Metal / DX12) for compute.
>
> Prebuilt binaries for Linux / macOS / Windows on the Releases page.
>
> Source: [repo]

---

## LONG — blog / writeup

> ## Galápagos: breeding pictures from formulas
>
> Galápagos is an interactive evolutionary art generator. It takes the idea Karl
> Sims demonstrated in 1991 — evolving mathematical expressions into images by
> picking the ones you like — and runs it on a modern GPU through Rust and wgpu.
>
> **Every image is a formula.** For each pixel, three independent expression trees
> produce three channels. Those channels are interpreted through a per-individual
> color model — HSV, RGB, HSL, CMY, or YUV — to become a final RGB color. The color
> model isn't fixed: it's an evolvable trait that rarely flips to a neighbor, so a
> strong lineage can slowly drift between color spaces as it breeds.
>
> **You are the fitness function.** The app shows a grid of 16 individuals. You
> select the ones you like and breed the next generation; the system recombines
> them with subtree crossover and mutates them across a set of 47 operators
> spanning terminals, unary, binary, and ternary arities. Repeat, and the
> population converges toward whatever you keep choosing.
>
> **How it runs.** Each genome is a strongly-typed expression tree that compiles to
> a 1024-instruction stack-machine program. The renderer uploads the whole
> population as a flat buffer and evaluates all 16 tiles in a single GPU compute
> dispatch. The WGSL shader's constants are generated from the Rust config at build
> time so the two halves stay in lockstep.
>
> **The widget.** A second binary is a small always-on-top window that animates a
> saved genome. It walks the genome's numeric values — constants and coordinate
> scales — within a bounded neighborhood of the seed, leaving the expression
> structure untouched, so the result deforms organically and every frame is a
> sharp, real genome rather than a dissolve between two images. It also has a
> seamless loop mode, where each value completes one full cycle over the loop so
> the animation returns exactly to its start — useful for recording.
>
> **Try it.** Prebuilt binaries for Linux, macOS (Intel and Apple Silicon), and
> Windows are on the Releases page; the bootstrap scripts will install Rust and
> build if you'd rather work from source. The project page has an interactive
> explainer that animates the whole pipeline — tree growth, pixel evaluation,
> coloring, channel remapping, and evolution.
>
> Project page: [project page]
> Source and binaries: [repo]
