---
title: ASTRA candidate publication proposal
slug: astra-release-proposal-2026-09-07
status: draft
tags: [release, beta, evidence]
audience: [human, agent]
layer: meta
created: 2026-09-07
updated: 2026-09-08
related: [[astra-execution-2026-09-07]], [[release-workflow]]
---

# Candidate publication proposal

**Current hold:** final onboarding MSI `cf8db2e0b41c…` passes its build, packaged
runtime check, native prerequisite regressions, browser journeys and Clippy.
A fresh three-task host rehearsal passed review/Apply, 11 repository tests,
26 independent checks, Cancel and receipt persistence after restart. Its new
52-second silent film and website evidence checks also pass.
Its 354-input digest is
`c721a49414e348dc93d7801c9d36fefdd445a75d44dd3ecaa6487824265e38f0`.
The preceding static CRT MSI `1f47f34d673b…` installed with exit 0 and rendered
its welcome screen through Start in the clean guest without the checked VC++
runtime DLLs. That session found the missing-Git guidance defect corrected in
the new candidate. Guest acceptance and hosted CI for the new bytes remain
pending. The current records are `astra-final-native-2026-09-08.json` and
`astra-final-demo-2026-09-08.json` under `tooling/benchmarks/results/`.
Earlier mission/film/Bench records retain their historical artifact identities.

This is a sequence for approval, not authorization or a release announcement.
Version 1.2.2 remains a local candidate on draft PR31's `f64a0b0` plus the
reviewed packaging and onboarding corrections. Build-time HEAD alone does not
identify the MSI. Public GitHub, npm and deployed downloads were
observed at 1.2.1 on September 7.

## CI and source

The September 7 resumption verified the owner's GitHub Team upgrade in Brave:
2,000 of 3,000 included Actions minutes used at 08:25 UTC, before the run, $0 billable Actions usage, and the
existing $0 Actions budget with stopping still enabled. The user instructed the
lead to continue and conserve Actions. The necessary consolidated PR31
corrections have now passed; no paid budget increase is authorized.
The first push completed CI run `34100785788`: eight passed and four failed,
from three causes now repaired and locally verified. Do not retry unrelated jobs
or run duplicate workflows merely to check billing.
The earlier proposed $10 overage allowance was not applied. Historical job times
at documented rates imply about $1.396 for CI and $2.482 for the release workflow,
excluding storage and duration changes. These are estimates. The failed run
34038203378 never executed its required checks; the generic billing annotation
does not establish a failed card.

All twelve jobs in run `34112393479` passed on the exact reviewed commit
`f64a0b092b02af917fea52180a4e76458e2f1934`, completed at 11:13:50 UTC.
The subsequent billing view showed 2,520/3,000 included minutes used, 480
remaining and $0 billable Actions usage. Recheck that allowance before an
approved publication run; it is not reserved. The Release workflow includes
another full CI gate, and merging main also triggers CI and website deployment.
Account for those actual triggers rather than treating publication as free.
Preserve the private master brief and unrelated untracked work outside commits
and packages. The PR remains draft while clean-install and publication gates
are unresolved; the earlier failed runs do not supersede this passing result.

On September 8 at 02:55 UTC, the read-only billing API still reported zero net
Actions usage and unchanged runner-minute quantities. The latest green run's
actual job intervals at the current reported SKU rates imply about $1.372 gross
cost equivalent for one comparable CI run, excluding storage and duration changes.
This is an estimate, not a charge or reserved allowance. Consolidate the runtime,
onboarding and evidence changes into one authorized PR-only run; keep the existing
$0 stopping budget and all required checks.

PR-only supersession cancellation and 60-minute Rust/native versus 30-minute
other-job limits bound wasted compute while retaining all checks. Reuse existing
caches; adding another large cache provides no first-run saving. A PR branch
push triggers CI only. A main merge also triggers the independent website
deployment workflow, so merge/publication approval remains necessary.

## Candidate and clean machine

Use `target/astra-clean-vm-20260907/final-packet/`; all sixteen entries
passed the archive and read-only media roundtrip checks. Its manifest identifies
MSI `cf8db2e0b41ceaaf678d3c74226b62dc62589f2a68880ec8ba49d42088dbd911`,
packaged EXE `d75be6b767164e681b278c717551b84f62baf6987467751f2e98a39538c72db5`,
source-input snapshot and toolchains. Development-host extraction establishes
narrower evidence than installation. `target/astra-ci-windows-validation-packet/`
and its MSI `614d44e2d47e…` are retained failed-candidate history.

The owner's subsequent “do it for me” authorizes assistant-owned disposable
Windows VM preparation and installer validation. Portable QEMU and WHPX firmware
execution are verified on the existing Home host without enabling host features.
A Windows 10 LTSC evaluation guest was prepared with a one-shot empty-disk
guard, verified read-only packet and explicit memory headroom. Continue its
existing installation; the previous replacement installed and rendered welcome
after removal of the failed candidate. The final onboarding MSI is staged next.
The full Windows media checksum passed; available RAM and commit capacity are
checked before boot and monitored during installation. This guest can
establish Windows 10-specific evidence; a development-host profile cannot, and
Windows 11 is a separate untested environment. See the current local provisioning
record in `target/astra-clean-vm-20260907/PROVISIONING.md`.

Follow the supplied guide and create a new actual result record without
rewriting the packet's historical template. Test Start-menu launch without source/dev PATH,
the optional-CLI path, real Review → Apply, hashes/checks, Stop, failure, drift,
restart and the claimed upgrade/uninstall behavior. Preserve repositories.
The full clean-install protocol remains unexecuted. Authenticate the vendor only
through its official guest flow under the applicable real-usage consent; do not
copy host credentials. Preserve global Windows protections.

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
