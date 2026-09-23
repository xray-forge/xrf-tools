import { Material, MeshBasicNodeMaterial } from "three/webgpu";

/** What a slot draws with in a pass that does not draw it, or whose surface is missing: nothing, and nothing to compile. */
export const HIDDEN_MATERIAL: Material = new MeshBasicNodeMaterial({ visible: false });
