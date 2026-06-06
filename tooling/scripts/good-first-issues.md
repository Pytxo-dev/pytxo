# Good first issues (create on GitHub)

Run from repo root with `gh` CLI after v0.1.0 tag:

```bash
gh issue create --title "Add Homebrew tap for pytxo" --label "good first issue,packaging" --body "Create a Homebrew formula that downloads GitHub Release binaries for pytxo v0.1.0+."
gh issue create --title "Add winget manifest for pytxo" --label "good first issue,packaging" --body "Submit a winget package manifest for pytxo-windows-x64.exe from GitHub Releases."
gh issue create --title "Docusaurus local search" --label "good first issue,docs" --body "Add @easyops-cn/docusaurus-search-local or Algolia DocSearch to apps/docs."
gh issue create --title "TUI: log tail panel" --label "good first issue,tui" --body "Add a log tail view in pytxo-tui when selecting an agent (integrate with pytxo-orchestrate logs API)."
gh issue create --title "npm: Windows ARM64 release asset" --label "help wanted,packaging" --body "Extend release.yml matrix and npm postinstall for pytxo-windows-arm64.exe."
```
