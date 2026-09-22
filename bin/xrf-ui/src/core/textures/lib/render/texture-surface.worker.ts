import { TTextureSurfaceRequest, TTextureSurfaceResponse } from "@/core/textures/lib/render/texture-surface-messages";
import { TextureSurfaceServer } from "@/core/textures/lib/render/texture-surface-server";
import { Logger } from "@/lib/logging";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Draws lit surfaces, on a thread of its own.
 */
const server: TextureSurfaceServer = new TextureSurfaceServer((response: TTextureSurfaceResponse): void =>
  self.postMessage(response)
);

self.onmessage = (event: MessageEvent<TTextureSurfaceRequest>): void => server.take(event.data);

log.info("Texture surface worker started");
