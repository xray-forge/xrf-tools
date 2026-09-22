/** Which thread a viewport's frames are drawn on. */
export enum ERenderThread {
  /** The one the interface runs on, where a slow frame is a slow interface. */
  MAIN = "main",
  /** One of its own, where a slow frame is only a slow picture. */
  WORKER = "worker",
}
