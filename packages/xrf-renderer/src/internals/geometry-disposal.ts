import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, InterleavedBufferAttribute, WebGPURenderer } from "three/webgpu";

import { destroyGeometryAttribute } from "#/internals/renderer-backend";

type TGeometryAttribute = BufferAttribute | InterleavedBufferAttribute;

/** Three's record of the geometries it drew, on its renderer, which its typings do not state. */
interface IRendererGeometries {
  _geometries: Nullable<{
    /** The dispose handler each drawn geometry was given, which frees through the first render object that drew it. */
    _geometryDisposeListeners: Map<BufferGeometry, () => void>;
    /** The line index three built for a geometry drawn as a wireframe. */
    wireframes: WeakMap<BufferGeometry, TGeometryAttribute>;
  }>;
}

/**
 * Disposes a geometry built over another's buffers, freeing only what it alone names. Three frees a disposed geometry's
 * buffers through the first render object that drew it, by that object's cached list of them: once that object went
 * (its material disposed, or its pipeline built again), the list still names every shared buffer, so three's handler
 * is taken off and this frees in its place.
 *
 * @param renderer - The renderer that drew it.
 * @param geometry - A part, or another geometry built over the source's buffers.
 * @param source - The geometry whose buffers it shares.
 */
export function disposeSharingGeometry(
  renderer: WebGPURenderer,
  geometry: BufferGeometry,
  source: BufferGeometry
): void {
  const geometries: IRendererGeometries["_geometries"] = (renderer as unknown as IRendererGeometries)._geometries;
  const onDispose: Maybe<() => void> = geometries?._geometryDisposeListeners.get(geometry);
  const shared: Set<Nullable<TGeometryAttribute>> = new Set([source.index, ...Object.values(source.attributes)]);

  if (geometries && onDispose) {
    const wireframe: Maybe<TGeometryAttribute> = geometries.wireframes.get(geometry);

    geometry.removeEventListener("dispose", onDispose);
    geometries._geometryDisposeListeners.delete(geometry);
    renderer.info.memory.geometries -= 1;

    if (wireframe) {
      destroyGeometryAttribute(renderer, wireframe);
    }
  }

  for (const attribute of [geometry.index, ...Object.values(geometry.attributes)]) {
    if (attribute && !shared.has(attribute)) {
      destroyGeometryAttribute(renderer, attribute);
    }
  }

  // Its render objects forget the buffers they listed.
  geometry.dispose();
}
