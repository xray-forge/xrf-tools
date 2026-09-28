import { Object3D } from "three/webgpu";

/** Three's `Object3D.dispose`, which its typings leave out. */
interface IDisposableObject {
  dispose(): void;
}

/**
 * Lets three forget what it built to draw an object. Its render objects keep the geometry, the material and the bundle
 * the object drew with, and go only on the object's own dispose event, or its material's: a geometry's dispose clears
 * their attribute list and leaves them, and with them every buffer the geometry named.
 *
 * @param object - An object nothing draws any more.
 */
export function disposeObject(object: Object3D): void {
  (object as unknown as IDisposableObject).dispose();
}
