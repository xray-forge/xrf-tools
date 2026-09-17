import { ArchiveShaderCompilerShader } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";

/**
 * What a surface costs the compiler to bake.
 *
 * @param shader - Shader to describe.
 * @returns Its lightmap density with the vertex lighting figures beside it.
 */
export function describeBaking(shader: ArchiveShaderCompilerShader): string {
  const density: string = formatNumber(shader.lightmapDensity, 2);
  const translucency: string = formatNumber(shader.vertexTranslucency, 2);
  const ambient: string = formatNumber(shader.vertexAmbient, 2);

  return `density ${density} · translucency ${translucency} · ambient ${ambient}`;
}

/**
 * What the compiler is told to do with a surface.
 *
 * @param shader - Shader to describe.
 * @returns Its flags, or a phrase for one that sets none at all.
 */
export function describeCompilerFlags(shader: ArchiveShaderCompilerShader): string {
  return shader.flags.length ? shader.flags.join(", ") : "Neither collides nor is drawn";
}
