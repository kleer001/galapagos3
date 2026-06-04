# Publicity plan

How to announce Galápagos releases without overselling them. Borrowed from the
funkworks publicity pipeline and trimmed to what this project actually needs: a
single Rust app with prebuilt binaries, aimed at the generative-art and Rust
communities. No DCC marketplaces, no newsletter infrastructure required.

The non-negotiable rule: every post passes [`honesty-checklist.md`](honesty-checklist.md)
before it goes out. Enthusiasm is fine; fabricated backstory and unbacked
superlatives are not.

## What we announce

Galápagos is a GPU-accelerated evolutionary art generator in Rust. It breeds
random typed expression trees into 2K images through human selection, inspired by
Karl Sims' 1991 work. Each tagged release ships prebuilt binaries for Linux,
macOS (Intel + Apple Silicon), and Windows on the
[Releases page](https://github.com/kleer001/galapagos3/releases), plus one-line
bootstrap install scripts. The build artifacts mean a reader can try it without a
Rust toolchain — lead with that in any post.

Verifiable hooks to draw from (all true, all in the README / source):
- Three expression trees → three channels → an evolvable per-individual color
  model (HSV / RGB / HSL / CMY / YUV) → RGB.
- 47 operators; subtree crossover plus two mutation modes.
- Genomes compile to 1024-instruction stack-machine bytecode; all 16 tiles render
  in a single wgpu compute dispatch.
- A second binary, the desktop widget, animates a saved genome by a *genetic
  walk* (sharp every frame, never a cross-dissolve) and a *seamless loop* mode.

## Assets to prepare

Visual-first communities scroll past text. Have these ready before posting:

- [ ] **A looping clip / GIF of the widget.** The seamless-loop mode exists for
      exactly this — a short clip that repeats with no visible seam. `docs/banner.gif`
      is an existing animated banner that can serve, or capture a fresh loop.
- [ ] **A few striking still renders** saved from the breeder (`S` on a tile) for
      a gallery / thumbnail.
- [ ] **A "Download" line in the top-level README** pointing at the Releases page,
      so readers arriving from a post find the binaries (the README currently leads
      with build-from-source).
- [ ] Confirm the project page (https://kleer001.github.io/galapagos3/) and its
      interactive genome explainer load — most posts link there.

## Where to post

| Venue | Tier | Angle | Link target |
|---|---|---|---|
| Hacker News (Show HN) | medium | Karl Sims lineage; expression trees → GPU art | project page |
| r/generative | short + clip | the visual first; "evolve your own, prebuilt binaries" | project page |
| r/proceduralgeneration | short + clip | same, framed around the generative method | project page |
| r/rust | medium | Rust + wgpu compute, bytecode VM on the GPU | repo |
| Mastodon / Bluesky | short | clip + tags (#generative #genuary #rustlang) | project page |

Notes:
- Posting is manual. Paste the matching tier from `announce.md`.
- HN Show HN title format: `Show HN: Galápagos – evolve math expression trees into 2K art on the GPU`.
- Lead image/clip matters more than the headline on the Reddit and social venues.
- Link announcements to the **project page**, not directly to a release zip — the
  page has context and download links. r/rust is the exception (link the repo).

## Workflow

1. Cut the release and confirm prebuilt binaries are attached (the release CI in
   `.github/workflows/release.yml` does this on every `v*` tag).
2. Prepare the assets above.
3. Draft / update [`announce.md`](announce.md) for the venues you're targeting.
4. **Run the honesty audit** ([`honesty-checklist.md`](honesty-checklist.md)) over
   `announce.md`. Resolve every flagged line. Do not post until it passes clean.
5. Post the matching tier to each venue, with the clip/stills.
6. Record what happened in the retrospective below.

## Retrospective

After a publicity push, note what worked and what to change next time — which
venue drove traffic, which framing landed, any venue rules that bit (image
requirements, title length caps, flair). Keep entries short: one observation,
one adjustment. This section is the project's running memory of what to repeat
and what to drop.

_(none yet)_
