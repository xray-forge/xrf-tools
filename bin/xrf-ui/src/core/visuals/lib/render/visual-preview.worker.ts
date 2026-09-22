import { TVisualPreviewRequest, TVisualPreviewResponse } from "@/core/visuals/lib/render/visual-preview-messages";
import { VisualPreviewServer } from "@/core/visuals/lib/render/visual-preview-server";
import { Logger } from "@/lib/logging";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Draws model previews, on a thread of its own.
 */
const server: VisualPreviewServer = new VisualPreviewServer((response: TVisualPreviewResponse): void =>
  self.postMessage(response)
);

self.onmessage = (event: MessageEvent<TVisualPreviewRequest>): void => server.take(event.data);

log.info("Visual preview worker started");
