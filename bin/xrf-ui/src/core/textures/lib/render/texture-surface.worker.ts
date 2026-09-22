import { RenderWorkerHost } from "@/core/render/lib/worker/render-worker-host";
import { TTextureSurfaceRequest } from "@/core/textures/lib/render/texture-surface-messages";
import { TextureSurfaceServer } from "@/core/textures/lib/render/texture-surface-server";
import { Logger } from "@/lib/logging";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Draws lit surfaces, on a thread of its own.
 */
const host: RenderWorkerHost<TTextureSurfaceRequest> = new RenderWorkerHost(
  (target, element) => new TextureSurfaceServer(target, element, (response) => self.postMessage(response)),
  (response) => self.postMessage(response)
);

self.onmessage = (event: MessageEvent<TTextureSurfaceRequest>): void => host.take(event.data);

log.info("Texture surface worker started");
