import { IDdsRefusal } from "@xrf/dds";

import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererReport } from "#/contract/renderer-report";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";

/**
 * What a consumer hands the renderer, and what it is told back.
 */
export interface IRendererClientOptions {
  /** The renderer's worker, from `createRendererWorker`. */
  worker: Worker;
  settings: IRendererSettings;
  onReady?: (device: IRendererDevice) => void;
  /** The renderer stopped for good, and why: every settle and capture waiting rejects, and every later one at once. */
  onFailed?: (reason: string) => void;
  onReport?: (report: IRendererReport) => void;
  /** A texture's file was refused as stored; putting its decoded picture under the same key fills the slot. */
  onTextureRefused?: (key: string, refusal: IDdsRefusal) => void;
  /** A texture the renderer fetched arrived or could not, for the put that asked for it: what it came to. */
  onTextureFetched?: (key: string, fetch: IRendererTextureFetch) => void;
}
