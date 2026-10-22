# pytxo (npm)

npm wrapper for the [Pytxo](https://pytxo.com) CLI. The first `pytxo` command downloads the same-version prebuilt binary from the public [pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) repository and checks it against the release's `SHA256SUMS.txt`. Package managers that run install scripts do this during `postinstall` instead; npm 12 and pnpm 10 skip it by default, and npm's warning about it is safe to ignore.

Community: [Discord](https://discord.gg/AUFRPFjSYv)

```bash
npm i -g pytxo
pytxo doctor
```

Set `PYTXO_SKIP_DOWNLOAD=1` and `PYTXO_BINARY=/path/to/pytxo` for local development.
