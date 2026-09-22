import { TLevelRenderRequest } from "@/core/level/lib/render/level-render-messages";
import { LevelRenderServer } from "@/core/level/lib/render/level-render-server";
import { RenderWorkerHost } from "@/core/render/lib/worker/render-worker-host";
import { Logger } from "@/lib/logging";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Draws levels, on a thread of its own.
 */
const host: RenderWorkerHost<TLevelRenderRequest> = new RenderWorkerHost(
  (target) => new LevelRenderServer(target, (response) => self.postMessage(response)),
  (response) => self.postMessage(response)
);

self.onmessage = (event: MessageEvent<TLevelRenderRequest>): void => host.take(event.data);

log.info("Render worker started");
