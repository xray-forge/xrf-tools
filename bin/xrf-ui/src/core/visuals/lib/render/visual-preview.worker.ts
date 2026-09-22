import { RenderWorkerHost } from "@/core/render/lib/worker/render-worker-host";
import { TVisualPreviewRequest } from "@/core/visuals/lib/render/visual-preview-messages";
import { VisualPreviewServer } from "@/core/visuals/lib/render/visual-preview-server";
import { Logger } from "@/lib/logging";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Draws model previews, on a thread of its own.
 */
const host: RenderWorkerHost<TVisualPreviewRequest> = new RenderWorkerHost(
  (target, element) => new VisualPreviewServer(target, element, (response) => self.postMessage(response)),
  (response) => self.postMessage(response)
);

self.onmessage = (event: MessageEvent<TVisualPreviewRequest>): void => host.take(event.data);

log.info("Visual preview worker started");
