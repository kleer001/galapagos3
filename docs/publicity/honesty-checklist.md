# Honesty checklist

Every line of announcement copy must pass this audit before it is posted
anywhere. Adapted from the `honest-copy` skill in the funkworks repo. The point
is simple: nothing in a public post should be fabricated, unverifiable, or
emotional filler. If a claim can't be traced to the code, the git history, the
Releases page, or something the author has actually said, it does not go out.

Run this against `announce.md` (and any other copy) before posting.

## The four tests

**1. First-person experience claims.**
Any sentence starting with "I", "I've", "Every time I", "I got tired of", etc.
Is it verifiably true from the git history or something the author has stated?
Authorship ("I made this", "I built this in Rust") is fine — the author is
posting. Fabricated *motivation* or *backstory* is not.

> Flag: "I got tired of dissolve-based art tools" (invented backstory)
> Keep: "I built this on wgpu compute shaders" (verifiable authorship)
> Keep: "Karl Sims demonstrated evolved expression art in 1991" (verifiable fact)

**2. Ordinal and superlative claims.**
"First release", "the only", "the most complete", "fastest", etc. Verify ordinals
against `gh release list` and the git history. This project is past its first
release — never call any post "my first release." Cut superlatives that aren't
backed by a measurement or a cited source.

**3. Implied repeated personal experience.**
"every time", "I kept having to", "I always ended up" implies something happened
repeatedly. Fabricated unless the author has said so. Rephrase as a description
of the mechanism in second or third person.

**4. Emotional / rhetorical filler.**
"which feels right", "finally", "at last", "a labor of love" — sentiment without
information. Flag and cut. Also cut stock metaphors ("tedious dance", "magic")
that hide the literal mechanism; describe what the software actually does.
("Friction" and "pain point" are acceptable plain terms.)

## Performance-claim rule (project-specific)

The widget shows a live frame-time HUD, and measured numbers exist for specific
genomes at specific settings on one machine — they are not general claims. Do not
put a specific fps or millisecond figure in a post unless it names the hardware
and settings it was measured at. Prefer architectural facts that are always true:
"renders all 16 tiles in a single GPU compute dispatch", "expression trees
compiled to stack-machine bytecode". Avoid bare "real-time" as a guarantee.

## Output format

For each flagged line:

```
LINE: [quote the sentence]
PROBLEM: [which test it fails and why]
FIX: [an honest replacement that says the same thing, or "cut it"]
```

If nothing is flagged, say so explicitly: "No issues found." Then apply the
agreed fixes before the copy is posted.
