/** The device, as far as staging buffers are made on it and let go. */
export interface IStagingDevice {
  createBuffer(descriptor: { label: string; size: number; usage: number }): { destroy(): void };
}
