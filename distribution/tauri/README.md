# Tauri updater signing

Generate a minisign keypair for Reality Deck auto-updates:

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/pytxo.key
```

1. Copy the printed **public key** into `apps/desktop/src-tauri/tauri.conf.json` → `plugins.updater.pubkey`.
2. Store the **private key** contents in GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`.
3. Optionally set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if the key is encrypted.

The Release workflow passes these secrets to `tauri-action` when building desktop installers.
