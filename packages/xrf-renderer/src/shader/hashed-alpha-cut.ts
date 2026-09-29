import { Node } from "three/webgpu";

import { SettingsUniforms } from "#/uniforms/settings-uniforms";

/** What a cut-out is cut by where a temporal resolve can average its cut over frames. */
export interface IHashedAlphaCut<T extends "vec4" | "float"> {
  /** The alpha the surface is cut by. */
  alpha: Node<"float">;
  /** What it writes where it stands. */
  output: Node<T>;
  /** Where it is cut. */
  reference: Node<"float">;
  /** Whether the resolve is temporal, and the frame. */
  settings: SettingsUniforms;
}
