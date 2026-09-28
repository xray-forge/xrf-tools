import { TRendererRequest } from "#/contract/renderer-request";
import { TRendererResponse } from "#/contract/renderer-response";
import { RendererHost } from "#/host/renderer-host";

const host: RendererHost = new RendererHost((response: TRendererResponse, transfer: Array<Transferable> = []) =>
  self.postMessage(response, { transfer })
);

self.onmessage = (event: MessageEvent<TRendererRequest>): void => host.take(event.data);
