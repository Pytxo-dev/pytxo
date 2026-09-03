# Pytxo demo treatment for Grok

## The one-sentence idea

An agent says the work is done. Pytxo proves what changed, asks for authority to
apply those exact bytes, and refuses when the checkout no longer matches the
review.

The film should feel like a forensic instrument under pressure. Keep it spare.
No robot mascots, floating code, fake terminals, holograms, or invented product
screens.

## Audience and wedge

The viewer runs coding agents against repositories and has felt the gap between
an agent's confident completion message and the state of the checkout. They are
usually a senior developer, staff engineer, engineering lead, or security-minded
operator.

Pytxo's wedge is a reviewed repository commit boundary for heterogeneous agents.
The film must make one difference obvious in the first 12 seconds: an agent's
claim and an observed effect are different things.

Headline options:

1. The agent said done. Pytxo checked.
2. Review the effect, not the promise.
3. Give agents room to work. Keep the commit boundary.

Use option 1 for the film. Use option 2 for a technical launch post. Option 3
fits the website hero or a longer product introduction.

## Format

- Hero cut: 75 seconds, 16:9, 3840x2160 master, 30 fps
- Social cut: 45 seconds, 9:16, 2160x3840, center-safe product crop
- Product cut: the existing 2 to 4 minute sequence in `DEMO.md`
- Captions: burned-in for social, optional sidecar for the hero cut
- Audio: restrained pulse, dry UI transients, one low impact at refusal
- Voice: calm operator, 145 to 155 words per minute, no trailer voice

The 75-second cut is the launch asset. The longer cut proves the workflow for
people who want to inspect it.

## Visual contract

Generated footage is connective material only. Product evidence comes from a
captured Pytxo build running the deterministic fixture.

Keep these rules locked:

- Product screens are screen recordings or stills from the release build.
- Do not ask Grok to render UI labels, hashes, code, terminal output, or receipts.
- Do not animate a claimed state into a verified state unless the captured UI
  and repository evidence show the transition.
- Do not imply cloud isolation when the receipt says worktree or overlay.
- Do not show rollback if the product only records an honest recovery path.
- Preserve the package digest, path count, run identity, and epistemic labels in
  product footage. Never replace them with cleaner fictional values.
- Generated interstitials use abstract physical metaphors: a gate, a measured
  boundary, a sealed package, and a mismatch that stops motion.

Color should follow the product: near-black graphite, cool metal, pale text,
chroma green for verified state, amber for authority, and red only for refusal.
Avoid neon cyberpunk. Light the frame like a precision lab, not a nightclub.

## 75-second storyboard

### 0:00 to 0:06 | The claim

Picture: black screen. A real terminal capture appears in a narrow left panel.
The agent completes and reports success. On the right, a clean repository status
capture does not yet contain the proposed change.

On-screen copy: `CLAIMED`

Narration: "The agent says the job is done. That is a claim."

Edit: hold for half a second after "claim." Let the contradiction register.

### 0:06 to 0:13 | The boundary

Picture: cut to Pytxo Work. Show the mission, Orbit permission profile, one
execution domain, and the running agent. Use a slow 3 percent push-in.

On-screen copy: `WORK HAPPENS OUTSIDE THE CHECKOUT`

Narration: "Pytxo gives the agent an isolated place to work and keeps the
checkout behind an approval boundary."

Generated bridge: a dark mechanical aperture closes around a bright packet of
light. Four seconds maximum. No text in generated footage.

### 0:13 to 0:22 | Independent observation

Picture: real Work and receipt captures. Show the verifier as a separate actor,
then show the test result and bounded receipt details.

On-screen copy changes from `CLAIMED` to `VERIFIED` only when the receipt is in
frame.

Narration: "A separate verifier runs with its own limits and receipt. Failure
stays failure."

Sound: one dry confirmation tick when `VERIFIED` appears.

### 0:22 to 0:34 | Exact review package

Picture: Run Review. Reveal the three affected paths, then the package digest.
Zoom only enough to make the digest and path count readable. Keep the rest of
the interface visible so the shot cannot be mistaken for a mockup.

On-screen copy: `3 PATHS. ONE PACKAGE DIGEST.`

Narration: "Pytxo prepares exact bytes for review. Three paths. One package
digest. Nothing reaches the repository yet."

Generated bridge: a physical evidence sleeve closes around three thin plates.
The sleeve receives a subtle engraved line, but no letters or numbers.

### 0:34 to 0:45 | Authority

Picture: click Apply. Show the consequence dialog that repeats the path count
and digest. Pause before clicking `Apply exact package`.

On-screen copy: `AUTHORITY IS EXPLICIT`

Narration: "Apply is a separate decision. The confirmation names the exact
package being authorized."

Sound: remove the music bed for the half-second before the click.

### 0:45 to 0:56 | Observed outcome

Picture: show the repository diff, the passing post-state test, and History
settled as committed. The order matters: checkout, test, ledger.

On-screen copy: `COMMITTED · VERIFIED · RECORDED`

Narration: "Only those reviewed bytes are applied. Pytxo checks the resulting
state and records what happened."

### 0:56 to 1:08 | Try to break it

Picture: hard cut. Repeat the review package setup from a second fixture run.
Before Apply, show a real operator edit to one affected path. Return to Pytxo
and apply. Show the stale-package refusal and the unchanged operator edit.

On-screen copy: `CHECKOUT CHANGED. APPLY REFUSED.`

Narration: "Change an affected file after review and the same package is no
longer safe. Pytxo refuses it. No partial write. No success receipt."

Generated accent: the evidence sleeve approaches the aperture, misaligns by a
few millimeters, and stops. It does not shatter or explode.

### 1:08 to 1:15 | Close

Picture: black field, Pytxo wordmark, then a final real History crop with one
committed run and one failed Apply attempt.

On-screen copy:

`PYTXO`

`THE COMMIT LAYER FOR AUTONOMOUS WORK`

`pytxo.com`

Narration: "Pytxo is the commit layer for autonomous work. Review the effect,
then authorize it."

## Grok generation prompts

Generate each bridge as a separate 4K 16:9 clip. Ask for six seconds, then use
the strongest two to four seconds in the edit. Grok should never generate the
product UI.

### Boundary aperture

> Macro cinematic shot inside a precision engineering lab. A compact packet of
> cool white light moves through a matte graphite channel toward a circular
> mechanical aperture. The aperture closes with measured, controlled motion and
> holds the packet outside a clean brushed-metal chamber. Near-black background,
> restrained chroma-green reflection, realistic materials, shallow depth of
> field, slow camera drift, 4K, 30 fps, premium industrial product film. No text,
> no logos, no people, no screens, no cyberpunk neon.

### Sealed evidence package

> Three thin precision-machined plates slide into a transparent evidence sleeve
> on a dark inspection table. The sleeve closes and a single fine verification
> line travels around its edge. Graphite, glass, pale metal, faint green status
> light, overhead laboratory lighting, locked camera with a subtle push-in,
> photorealistic product cinematography, 4K, 30 fps. No text, numbers, hashes,
> logos, hands, interfaces, fantasy technology, or glowing code.

### Mismatch refusal

> A sealed transparent evidence sleeve approaches a precisely fitted mechanical
> gate in a dark engineering lab. The alignment is off by only a few millimeters.
> A restrained red indicator appears and the gate stops the sleeve without
> damage. Nothing crosses the boundary. Realistic inertia, quiet controlled
> motion, matte graphite and glass, macro lens, 4K, 30 fps. No explosion, sparks,
> alarms, text, logos, screens, robots, or cyberpunk styling.

Universal negative prompt:

> illegible UI, fake code, gibberish text, floating windows, humanoid robots,
> hooded hacker, blue neon city, excessive bloom, lens dirt, glitch transitions,
> rapid camera movement, magic particles, explosion, destruction, stock footage

## Capture list from the release build

Use the exact success and stale-refusal fixtures documented in `DEMO.md`.
Record at 1600x1000 or higher with 100 percent display scaling and the dark
release theme.

Required product plates:

1. Work with the mission, agent, domain, wave, and Orbit receipt visible.
2. Verification actor receipt with a passing result.
3. Run Review with exactly three paths and the package digest.
4. Apply confirmation with the same path count and digest.
5. Repository diff followed by the post-state test.
6. History showing the committed success.
7. Operator edit to an affected path before the second Apply.
8. Stale refusal plus repository proof that only the operator edit remains.
9. History showing the failed Apply attempt without a success receipt.

Record mouse movement once and use it. Do not recreate cursors in post. Leave
enough handles around each action for a clean edit.

## Narration script

"The agent says the job is done. That is a claim.

Pytxo gives the agent an isolated place to work and keeps the checkout behind
an approval boundary.

A separate verifier runs with its own limits and receipt. Failure stays failure.

Pytxo prepares exact bytes for review. Three paths. One package digest. Nothing
reaches the repository yet.

Apply is a separate decision. The confirmation names the exact package being
authorized.

Only those reviewed bytes are applied. Pytxo checks the resulting state and
records what happened.

Change an affected file after review and the same package is no longer safe.
Pytxo refuses it. No partial write. No success receipt.

Pytxo is the commit layer for autonomous work. Review the effect, then authorize
it."

## Five launch gallery frames

Export each frame at 1270x760. Product plates must remain readable at that size.

1. `The agent said done. Pytxo checked.` Split the agent claim against the Work
   receipt. Keep `CLAIMED` and `VERIFIED` visually distinct.
2. `Work happens outside the checkout.` Show Orbit isolation and one execution
   domain. Caption the actual receipt mechanism.
3. `Review exact bytes.` Show three paths and the package digest from Run Review.
4. `Apply is a separate decision.` Show the consequence confirmation, with the
   exact path count and digest visible.
5. `Stale means stop.` Show the refusal beside proof that the operator's edit
   survived and no partial write occurred.

## Edit notes

Use hard cuts when truth changes: claimed to verified, prepared to authorized,
and safe to stale. Dissolves are allowed only between abstract bridges and real
product footage. Avoid speed ramps on UI.

Typography should match Pytxo's current site. Use one type size for statements
and a smaller monospaced line for evidence. Keep overlays out of the product's
own labels. Never cover the receipt, digest, path count, status, or run identity.

Mix the voice close and dry. The music should stay below the narration and lose
energy before Apply. The refusal gets one low mechanical stop, not a cinematic
boom.

## If a take fails

- If the provider is slow, use the deterministic fixture. Do not cut around a
  hanging live agent and imply completion.
- If Apply refuses unexpectedly, keep recording and inspect the stale-path
  reason. That footage may become the failure sequence, but never label it as
  the success take.
- If a hash or path count differs, recapture every downstream plate from that
  run. Do not splice evidence from separate runs.
- If Grok renders text or UI, discard the clip. Use a black-frame transition or
  the real product plate instead.
- If licensed narration or music is unavailable, publish the captioned silent
  cut. Do not use provisional audio.

## Recording checklist

- Checkout is clean except for the fixture's expected state.
- Release build, CLI version, source commit, and installer checksum are logged.
- Success and refusal use separate run IDs and package digests.
- The receipt mechanism matches the actual platform.
- No API keys, usernames, unrelated repositories, or notifications are visible.
- Captions match the final voice recording.
- Final frame links to `pytxo.com`, not a private repository.
- Watch the export once at normal speed with no narration. The state changes
  should still make sense.
- Watch once muted and once on a phone-sized crop.
- Verify every public claim against the captured run before publishing.

## Short description

An agent can report success without changing the state you care about. Pytxo
runs the work behind a repository boundary, verifies the result, prepares exact
bytes for review, and applies them only after explicit approval. If an affected
file changes after review, Apply is refused and the failed attempt is recorded.

## Technical launch article angle

Title: `The agent said done. The checkout disagreed.`

Open with the stale-package case, not a feature list. Walk through the five
state changes that matter: proposed work, isolated execution, independent
verification, explicit authorization, and observed post-state. Include the two
run IDs, both package digests, the affected-path edit, and the final repository
assertions from the release rehearsal. Close with the current boundary: one
repository root, Orbit or Galaxy, reviewed Apply, Windows-first Desktop.
