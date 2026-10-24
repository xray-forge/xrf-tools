// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { IIpcBulkRouteSignature } from "@/core/ipc/signature";

/** Every bulk route of the `levels` plugin, as the transport serves it. */
export enum ELevelsBulkRoute {
  READ_SCREENSHOT = "levels/read_screenshot",
}

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const levelsBulkRoutes = {
  readScreenshot: (path: string): IBulkCall => ({ args: { path }, route: ELevelsBulkRoute.READ_SCREENSHOT }),
};

/** What each bulk route of the `levels` plugin takes, keyed by its route. */
export interface ILevelsBulkRouteSignatures {
  [ELevelsBulkRoute.READ_SCREENSHOT]: IIpcBulkRouteSignature<typeof levelsBulkRoutes.readScreenshot, ["path"]>;
}
