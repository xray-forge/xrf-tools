import { TLevelRenderRequest, TLevelRenderResponse } from "@/core/level/lib/render/level-render-messages";
import { LevelRenderServer } from "@/core/level/lib/render/level-render-server";
import { Logger } from "@/lib/logging";

const log: Logger = new Logger(__MODULE_NAME__);

/**
 * Draws levels, on a thread of its own.
 */
const server: LevelRenderServer = new LevelRenderServer((response: TLevelRenderResponse): void =>
  self.postMessage(response)
);

self.onmessage = (event: MessageEvent<TLevelRenderRequest>): void => server.take(event.data);

// The one line proving a second thread ran this module at all, which nothing on the page can say for it.
log.info("Render worker started");
