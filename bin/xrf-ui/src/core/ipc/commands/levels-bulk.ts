// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const levelsBulkRoutes = {
  readScreenshot: (path: string): IBulkCall => ({ args: { path }, route: "levels/read_screenshot" }),
};
