---
title: ASTRA candidate publication proposal
slug: astra-release-proposal-2026-09-07
status: draft
tags: [release, beta, evidence]
audience: [human, agent]
layer: meta
created: 2026-09-07
updated: 2026-09-07
related: [[astra-execution-2026-09-07]], [[release-workflow]]
---

# Candidate publication proposal

This is a sequence for approval, not authorization or a release announcement.
Version 1.2.2 remains a local candidate descended from draft PR31 at
`ff0b88fb71224d579267e697261ae4d831f0cd76`. Uncommitted improvements must be
reviewed and included in the final source identity; that old HEAD alone does
not identify the new build. Public GitHub, npm and deployed downloads were
observed at 1.2.1 on September 7.

## CI and source

The September 7 resumption verified the owner's GitHub Team upgrade in Brave:
2,000 of 3,000 included Actions minutes used, $0 billable Actions usage, and the
existing $0 Actions budget with stopping still enabled. The user instructed the
lead to continue and conserve Actions. Use the remaining included allowance for
one consolidated PR31 candidate push; no paid budget increase is needed now.
The earlier proposed $10 overage allowance was not applied. Historical job times
at documented rates imply about $1.396 for CI and $2.482 for the release workflow,
excluding storage and duration changes. These are estimates. The failed run
34038203378 never executed its required checks; the generic billing annotation
does not establish a failed card.

Run CI on the exact reviewed commit. Keep the PR draft until the required jobs
execute and pass. Do not
rerun the old HEAD and label it final-source coverage. Preserve the private
master brief and unrelated untracked work outside commits and packages.

PR-only supersession cancellation and 60-minute Rust/native versus 30-minute
other-job limits bound wasted compute while retaining all checks. Reuse existing
caches; adding another large cache provides no first-run saving. A PR branch
push triggers CI only. A main merge also triggers the independent website
deployment workflow, so merge/publication approval remains necessary.

## Candidate and clean machine

Use the local `target/astra-windows-validation-packet/` after its inventory and
hash checks pass. Its manifest identifies the exact MSI, packaged executable,
source-input snapshot and toolchains. Development-host extraction and native
automation establish narrower evidence than installation.

The owner provides an approved clean Windows x64 machine or disposable VM on a
separate adequately resourced host, with permission for installer/admin actions,
prerequisites and vendor authentication/usage. Follow the supplied guide and
fill the result template. Test Start-menu launch without source/dev PATH,
the optional-CLI path, real Review → Apply, hashes/checks, Stop, failure, drift,
restart and the claimed upgrade/uninstall behavior. Preserve repositories.
The current Windows Home development host cannot establish this gate.

## Specific publication proposal

Once actual evidence is attached, request approval for the final PR31 merge
commit, tag `v1.2.2`, private and public GitHub release assets, `pytxo@1.2.2`
npm publication, distribution README/install-script synchronization, and the
matching website deployment. These are separate external effects. The existing
Release workflow publishes; it is not a harmless artifact-only preview.

Use two explicit publication approvals with the current workflow. First, after
final-source CI and an approved merge reachable from main, approve one
**private-only** workflow dispatch: version `1.2.2`, `mirror_public=false`,
`publish_npm=false`, and the separately agreed signing choice. This still creates
a private GitHub release and consumes CI allowance. Download its exact assets
into fresh staging and complete clean-machine acceptance of that built MSI.
The workflow has no pause between artifact preparation and private publication.
Use `workflow_dispatch` for this first approval; do not push a `v*` tag as a
substitute. The tag-triggered workflow enables public mirroring and npm
publication, so it exceeds a private-only approval.

Second, approve promotion of those exact validated assets to the public release,
plus the npm wrapper and distribution files from the same source commit, then
the site. Promotion must upload the staged bytes directly; **do not rerun the
Release workflow**, which would rebuild them. Compare private, staged and public
hashes. If a private-only run is not approved, keep the candidate local; adding
a separate artifact-only workflow is an alternative, not a prerequisite invented
for this pass.

Expected CLI inventory: Linux x64/arm64, macOS x64/arm64, Windows x64, and
`SHA256SUMS.txt`. Desktop inventory: Windows x64 MSI and
`DESKTOP_SHA256SUMS.txt`; a configured updater-signed release also requires its
Windows updater asset and `latest.json`. Validate actual bytes, not just names.
Windows Authenticode and Tauri updater signatures are separate properties.

Recommend promoting GitHub Latest only with the tested updater-signed manifest.
If that is unavailable, keep the candidate local until an explicit manual-only,
non-Latest publication is approved and supported by the publication command.
The current workflow has no manual-only channel switch. Do not run it expecting
an unsigned Latest release to retain the previous release's manifest.

CI can rebuild source-equivalent artifacts with different hashes. Freeze and
revalidate the actual intended published MSI; never substitute an earlier tested
binary under its identity. Deploy website versioned download links only after
their matching public assets exist.

## Delivery and recovery

After authorized publication, anonymously fetch documented downloads and npm
metadata, follow redirects, validate binary content and hashes, then install the
downloaded MSI in the clean environment. Test the actual supported update route.
Only then label PUBLIC DOWNLOAD VERIFIED.

For a failed publication, stop subsequent deployment and record which writes
completed. Retain immutable versioned assets and manifests. An owner-approved
website rollback can restore 1.2.1 links; npm dist-tag changes and GitHub Latest
changes need their own approval. Do not overwrite 1.2.2 bytes or promise database
downgrade compatibility. Preserve repositories and back up local state before
any approved older-version installation. A corrective release uses a new version.

Private vulnerability reporting on the public distribution repository also
requires an owner decision: enable the existing GitHub private reporting feature
or provide a real supported private contact. Ordinary public issues must exclude
sensitive reports. No account or security settings were changed in this pass.
