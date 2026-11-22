import * as fs from "node:fs";
import { type ServerResponse } from "node:http";
import * as path from "node:path";

import { type Connect, type Plugin, type PreviewServer, type ViteDevServer } from "vite";

/** One Content Security Policy as Tauri's configuration writes it: a header value, or each directive's sources. */
type TTauriCsp = string | Record<string, string | Array<string>>;

/** The application's Tauri configuration, whose `app.security.csp` is the release policy. */
const TAURI_CONFIG: string = path.resolve(import.meta.dirname, "../../../app/tauri.conf.json");

/**
 * @param sources - A directive's sources, as one string or a list.
 * @returns Each source on its own.
 */
function splitSources(sources: string | ReadonlyArray<string>): Array<string> {
  return (typeof sources === "string" ? [sources] : [...sources]).flatMap((it) => it.split(/\s+/)).filter(Boolean);
}

/**
 * @param policy - A policy as Tauri's configuration writes it.
 * @returns Each directive's sources, in the policy's order.
 */
function readDirectives(policy: TTauriCsp): Map<string, Array<string>> {
  if (typeof policy !== "string") {
    return new Map(Object.entries(policy).map(([directive, sources]) => [directive, splitSources(sources)]));
  }

  return new Map(
    policy
      .split(";")
      .map((directive: string) => splitSources(directive))
      .filter((parts: Array<string>) => parts.length > 0)
      .map(([directive, ...sources]) => [directive, sources])
  );
}

/**
 * The release policy as the dev server sends it, with what Vite itself needs added: its inline module preamble and its
 * hot-reload socket on the port it serves.
 *
 * @param policy - The release policy, as Tauri's configuration writes it.
 * @param port - The port the server serves on.
 * @returns The development policy as a header value: `directive sources; directive sources`.
 */
export function toDevCsp(policy: TTauriCsp, port: number): string {
  const directives: Map<string, Array<string>> = readDirectives(policy);

  for (const [directive, source] of [
    ["script-src", "'unsafe-inline'"],
    ["connect-src", `ws://localhost:${port}`],
  ]) {
    // A directive the policy leaves out falls back to `default-src`, so it starts from that one's sources.
    const sources: Array<string> = directives.get(directive) ?? [...(directives.get("default-src") ?? [])];

    directives.set(directive, sources.includes(source) ? sources : [...sources, source]);
  }

  return [...directives].map(([directive, sources]) => [directive, ...sources].join(" ")).join("; ");
}

/**
 * Sends the release policy, with what Vite needs added, on everything the dev and preview servers serve.
 *
 * Tauri sets the policy only on assets it serves through its own protocol, and on desktop it loads a `devUrl` page
 * straight from the dev server, so without this a page served by Vite runs under no policy and a violation shows first
 * in a release build.
 *
 * @returns The plugin.
 */
export function devCspPlugin(): Plugin {
  function apply(server: ViteDevServer | PreviewServer, port: number): void {
    const config: { app?: { security?: { csp?: TTauriCsp } } } = JSON.parse(fs.readFileSync(TAURI_CONFIG, "utf8"));
    const policy = config.app?.security?.csp;

    if (!policy) {
      return;
    }

    const header: string = toDevCsp(policy, port);

    server.middlewares.use((_: Connect.IncomingMessage, response: ServerResponse, next: Connect.NextFunction) => {
      response.setHeader("Content-Security-Policy", header);
      next();
    });
  }

  return {
    name: "xrf-dev-csp",
    configureServer: (server: ViteDevServer) => apply(server, server.config.server.port),
    configurePreviewServer: (server: PreviewServer) => apply(server, server.config.preview.port),
  };
}
