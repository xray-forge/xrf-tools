import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, InterleavedBufferAttribute, WebGPURenderer } from "three/webgpu";

/**
 * The part of three's backend the renderer reads, which its typings do not state.
 */
export interface IRendererBackend {
  isWebGPUBackend?: boolean;
  device?: {
    features?: Iterable<string>;
    adapterInfo?: { vendor?: string; architecture?: string };
    limits?: { maxStorageBufferBindingSize?: number; maxBufferSize?: number; maxTextureArrayLayers?: number };
    queue?: { onSubmittedWorkDone?(): Promise<void> };
  };
  /** Whether each pass begins and ends with a timestamp; three reads it as every pass begins. */
  trackTimestamp?: boolean;
  hasTimestampQuery?(uid: string): boolean;
  getTimestamp?(uid: string): number;
  has?(object: object): boolean;
  destroyAttribute?(attribute: BufferAttribute): void;
}

/** The record of every attribute three uploaded, on its renderer, which its typings do not state. */
interface IRendererAttributes {
  _attributes: Nullable<{ delete(attribute: BufferAttribute | InterleavedBufferAttribute): unknown }>;
}

/**
 * Turns the GPU timing of every pass on or off from the next pass. Three decides it once, as the device opens, from the
 * renderer's `trackTimestamp` and the device's `timestamp-query`; turning it back on needs both.
 *
 * @param renderer - A renderer opened with `trackTimestamp`, on a device that times.
 * @param isTimed - Whether passes are timed from now on.
 */
export function setRendererTimestamps(renderer: WebGPURenderer, isTimed: boolean): void {
  getRendererBackend(renderer).trackTimestamp = isTimed;
}

/**
 * @param renderer - A renderer, its device open.
 * @returns Settles once the GPU has done everything submitted to it so far; at once for a backend that cannot say.
 */
export function whenSubmittedWorkDone(renderer: WebGPURenderer): Promise<void> {
  return getRendererBackend(renderer).device?.queue?.onSubmittedWorkDone?.() ?? Promise.resolve();
}

/** WebGPU's default `maxStorageBufferBindingSize`, which a device three opens without asking for more has. */
export const DEFAULT_STORAGE_LIMIT: number = 1 << 27;

/**
 * @param renderer - A renderer.
 * @returns Its backend, as far as it is read.
 */
export function getRendererBackend(renderer: WebGPURenderer): IRendererBackend {
  return renderer.backend as unknown as IRendererBackend;
}

/**
 * @param renderer - A renderer, its device open.
 * @returns Bytes one storage buffer may hold and be bound whole: the lesser of the device's two limits on it.
 */
export function toStorageLimit(renderer: WebGPURenderer): number {
  const limits: Maybe<{ maxStorageBufferBindingSize?: number; maxBufferSize?: number }> =
    getRendererBackend(renderer).device?.limits;

  return Math.min(
    limits?.maxStorageBufferBindingSize ?? DEFAULT_STORAGE_LIMIT,
    limits?.maxBufferSize ?? DEFAULT_STORAGE_LIMIT
  );
}

/** WebGPU's default `maxTextureArrayLayers`, which a device three opens without asking for more has. */
export const DEFAULT_ARRAY_LAYERS: number = 256;

/**
 * @param renderer - A renderer, its device open.
 * @returns Layers one texture array may hold on its device.
 */
export function toArrayLayerLimit(renderer: WebGPURenderer): number {
  return getRendererBackend(renderer).device?.limits?.maxTextureArrayLayers ?? DEFAULT_ARRAY_LAYERS;
}

/**
 * Frees the GPU buffer behind a storage attribute nothing binds any more, where it ever reached the GPU. Three frees a
 * geometry's attributes with the geometry and has no way to free a storage attribute on its own; the GPU finishes
 * whatever used it first.
 *
 * @param renderer - The renderer that uploaded it.
 * @param attribute - The attribute, replaced everywhere it was bound.
 */
export function destroyStorageAttribute(renderer: WebGPURenderer, attribute: BufferAttribute): void {
  const backend: IRendererBackend = getRendererBackend(renderer);

  // One replaced again before anything drew with it was never uploaded, and three's destroy assumes it was.
  if (backend.has?.(attribute)) {
    backend.destroyAttribute?.(attribute);
  }
}

/**
 * Frees the GPU buffer behind a vertex or index attribute no drawn geometry names, as three frees a disposed geometry's:
 * through its record of what it uploaded, so one never uploaded or freed already is passed over.
 *
 * @param renderer - The renderer that uploaded it.
 * @param attribute - The attribute.
 */
export function destroyGeometryAttribute(
  renderer: WebGPURenderer,
  attribute: BufferAttribute | InterleavedBufferAttribute
): void {
  (renderer as unknown as IRendererAttributes)._attributes?.delete(attribute);
}
