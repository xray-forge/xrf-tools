import { WebGPURenderer } from "three/webgpu";

import { compileSideBySide } from "#/internals/side-by-side-compile";
import { compileInto } from "#/pass/compile-into";
import { IObjectCompile } from "#/pass/object-compile";

/** Objects compiled at once, so the GPU process makes as many pipelines at a time on its threads. */
const COMPILE_LANES: number = 8;

/**
 * Compiles objects side by side, each lane taking the next as it finishes one: three compiles a scene one object at a
 * time and waits for each one's pipeline before the next, which the GPU process then makes one after another.
 *
 * @param renderer - The renderer compiling.
 * @param compiles - What to compile, taken one at a time as lanes free: it may stop early, or read what changed since.
 * @returns Settles once every lane ran out, or rejects with the first failure once they all have.
 */
export function compileObjects(renderer: WebGPURenderer, compiles: Iterator<IObjectCompile>): Promise<void> {
  async function lane(): Promise<void> {
    for (let next: IteratorResult<IObjectCompile> = compiles.next(); !next.done; next = compiles.next()) {
      const { target, object, scene, camera }: IObjectCompile = next.value;

      await compileInto(renderer, target, object, camera, scene);
    }
  }

  return compileSideBySide(
    renderer,
    Array.from({ length: COMPILE_LANES }, () => lane)
  );
}
