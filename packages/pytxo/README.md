# pytxo (npm)

npm wrapper for the [Pytxo](https://pytxo.com) CLI. Downloads the matching prebuilt binary from the public [pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) repo on `postinstall`.

```bash
npm i -g pytxo
pytxo doctor
```

Set `PYTXO_SKIP_DOWNLOAD=1` and `PYTXO_BINARY=/path/to/pytxo` for local development.
