// A local stand-in for Pytxo's update feed, for update-proof.ps1 on a disposable
// runner whose hosts file points the feed hosts here. Serves latest.json for the
// given installer and its updater signature, then the installer itself, and logs
// every request so the proof shows what the installed app actually fetched.
//
//   node update-feed.mjs --pfx <cert.pfx> --passphrase <pfx passphrase> --msi <installer> --version <x.y.z> --log <file>
import { appendFileSync, createReadStream, readFileSync, statSync } from "node:fs";
import https from "node:https";
import path from "node:path";

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
const name = path.basename(args.msi);
const manifest = JSON.stringify({
  version: args.version,
  notes: "Pytxo update proof (disposable runner)",
  pub_date: new Date().toISOString(),
  platforms: { "windows-x86_64": { signature: readFileSync(`${args.msi}.sig`, "utf8").trim(), url: `https://raw.githubusercontent.com/pytxo-update-proof/${encodeURIComponent(name)}` } },
});
const log = (line) => appendFileSync(args.log, `${new Date().toISOString()} ${line}\n`);

https.createServer({ pfx: readFileSync(args.pfx), passphrase: args.passphrase }, (request, response) => {
  log(`${request.method} ${request.headers.host}${request.url}`);
  if (request.url.endsWith("/latest.json")) {
    response.writeHead(200, { "content-type": "application/json" }).end(manifest);
  } else if (decodeURIComponent(request.url).endsWith(`/pytxo-update-proof/${name}`)) {
    response.writeHead(200, { "content-type": "application/octet-stream", "content-length": statSync(args.msi).size });
    createReadStream(args.msi).pipe(response);
  } else {
    response.writeHead(404).end();
  }
}).listen(443, "127.0.0.1", () => log(`listening, offering ${args.version} (${name})`));
