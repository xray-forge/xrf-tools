import { Nullable } from "@xrf/types";

/**
 * The limits the renderer raises from WebGPU's defaults to what the adapter supports: storage buffers a culling pass
 * binds at once, and how large the growable pools may become.
 */
const RAISED_LIMITS = ["maxStorageBuffersPerShaderStage", "maxStorageBufferBindingSize", "maxBufferSize"] as const;

/**
 * Asks for the adapter three will ask for, the same options, and reads the limits to open the device with. A device
 * opens with WebGPU's minimum limits unless it asks for more, whatever the adapter offers.
 *
 * @returns The limits to require of the device, or null where there is no adapter, for three to fail on itself.
 */
export async function getRendererDeviceLimits(): Promise<Nullable<Record<string, number>>> {
  // Three's own options; the DOM library does not know `featureLevel` yet.
  const options: GPURequestAdapterOptions & { featureLevel: string } = { featureLevel: "compatibility" };
  const adapter: Nullable<GPUAdapter> =
    typeof navigator === "undefined" ? null : await navigator.gpu.requestAdapter(options);

  if (!adapter) {
    return null;
  }

  return Object.fromEntries(RAISED_LIMITS.map((name) => [name, adapter.limits[name]]));
}
