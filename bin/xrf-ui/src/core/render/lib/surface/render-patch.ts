import { Material, WebGLProgramParametersWithUniforms } from "three";

/**
 * One edit to the shader three.js compiled for a material, in the shader's own source.
 */
export type TRenderPatch = (shader: WebGLProgramParametersWithUniforms) => void;

/**
 * What one patch is called on a material, and what it is cached under.
 */
export interface IRenderPatchIdentity {
  /** Identifies the patch: applying the same name again replaces it rather than adding a second. */
  name: string;
  /** What the program is cached under, the name unless the patch compiles a value into the shader. */
  key?: string;
}

interface IRenderPatchEntry extends IRenderPatchIdentity {
  patch: TRenderPatch;
}

/**
 * What each material has been patched with, in the order it was applied.
 */
const patched: WeakMap<Material, Array<IRenderPatchEntry>> = new WeakMap();

/**
 * Adds one shader patch to a material, or replaces the one of that name it already carries.
 *
 * @param material - Material to patch.
 * @param identity - What the patch is called, and what its program is cached under.
 * @param patch - The edit itself.
 */
export function applyRenderPatch(material: Material, identity: IRenderPatchIdentity, patch: TRenderPatch): void {
  const entries: Array<IRenderPatchEntry> = install(material);
  const at: number = entries.findIndex((entry: IRenderPatchEntry) => entry.name === identity.name);
  const entry: IRenderPatchEntry = { ...identity, patch };

  // Replaced in place rather than appended, because a patch closes over what it was built with - a texture, a
  // constant - and a surface re-dressed from one to another would otherwise carry both and compile the older one.
  if (at === -1) {
    entries.push(entry);
  } else {
    entries[at] = entry;
  }

  // Set here rather than by every caller: a patch added to a material that has already drawn changes its program.
  material.needsUpdate = true;
}

/**
 * Takes one patch off a material, leaving the rest of them.
 *
 * @param material - Material to unpatch.
 * @param name - What the patch was called.
 */
export function removeRenderPatch(material: Material, name: string): void {
  const entries: Array<IRenderPatchEntry> | undefined = patched.get(material);
  const at: number = entries?.findIndex((entry: IRenderPatchEntry) => entry.name === name) ?? -1;

  if (!entries || at === -1) {
    return;
  }

  entries.splice(at, 1);

  material.needsUpdate = true;
}

/**
 * The material's patch list, hooking the material up to it the first time one is asked for.
 *
 * @param material - Material being patched.
 * @returns Its list, which the hooks read every compile rather than closing over a copy.
 */
function install(material: Material): Array<IRenderPatchEntry> {
  const existing: Array<IRenderPatchEntry> | undefined = patched.get(material);

  if (existing) {
    return existing;
  }

  const entries: Array<IRenderPatchEntry> = [];

  patched.set(material, entries);

  material.onBeforeCompile = (shader: WebGLProgramParametersWithUniforms): void => {
    for (const entry of entries) {
      entry.patch(shader);
    }
  };

  material.customProgramCacheKey = (): string =>
    entries.map((entry: IRenderPatchEntry) => entry.key ?? entry.name).join("|");

  return entries;
}
