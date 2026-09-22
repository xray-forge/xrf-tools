import { TRendererRequest } from "#/contract/renderer-messages";
import { RendererHost } from "#/host/renderer-host";

const host: RendererHost = new RendererHost((response, transfer = []) => self.postMessage(response, { transfer }));

self.onmessage = (event: MessageEvent<TRendererRequest>): void => host.take(event.data);
