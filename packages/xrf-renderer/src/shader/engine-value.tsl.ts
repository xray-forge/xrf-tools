import { float, mix } from "three/tsl";
import { Node } from "three/webgpu";

import { IEngineValue } from "#/shader/engine-value";
import { EngineUniforms } from "#/uniforms/engine-uniforms";

/**
 * @param engine - The engine the scene is drawn as.
 * @param value - What each engine's shader writes.
 * @returns The one the scene is drawn as writes.
 */
export function toEngineValue(engine: EngineUniforms, value: IEngineValue): Node<"float"> {
  return mix(float(value.vanilla), float(value.extended), engine.extended);
}

/**
 * @param engine - The engine the scene is drawn as.
 * @returns Whether it is the extended engine.
 */
export function isExtendedEngine(engine: EngineUniforms): Node<"bool"> {
  return engine.extended.greaterThan(0.5);
}
