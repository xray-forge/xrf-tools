// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  AssetTextureDescriptor,
  AssetWorldSpec,
  SelectedVisualDescription,
  VisualSource,
} from "@/core/bindings/types/xrf-app";
import { VisualDependencies, VisualDescription } from "@/core/bindings/types/xrf-visual";

/** Commands */
export const visualsCommands = {
  /**
   * Stop browsing, leaving whatever visual is open on screen.
   *
   * The mounted sources stay: they belong to the shared asset world, which outlives any one session and is what makes
   * browsing the same root again free.
   */
  closeBrowse: () => __TAURI_INVOKE<null>("plugin:visuals|close_browse"),
  /** Drop the selected visual and its packed geometry. */
  closeModel: () => __TAURI_INVOKE<null>("plugin:visuals|close_model"),
  /**
   * The world the viewer was browsing, or null when it was showing one visual on its own.
   *
   * The rehydration probe for the tree, beside the one the selection already has: a reloaded frontend asks what is
   * being browsed and lists it again, so the panel comes back rather than emptying beside a model still open.
   */
  getBrowse: () =>
    __TAURI_INVOKE<{
      /** Asset whose own X-Ray root and installation are searched first, when the world is centred on one. */
      asset: string | null;
      /** Roots searched after the asset's own, in the order given. */
      roots: Array<string>;
    } | null>("plugin:visuals|get_browse"),
  /**
   * What the viewer had selected, or null when nothing is open.
   *
   * This is the rehydration probe: a reloaded frontend asks what is selected and then asks for that
   * source's geometry, so the selection survives a reload without the frontend storing anything.
   */
  getModel: () =>
    __TAURI_INVOKE<{
      source: VisualSource;
      /** The world the selection was opened in, so a reloaded frontend asks for geometry the same way. */
      world: AssetWorldSpec;
      description: VisualDescription;
      dependencies: VisualDependencies;
      /** What each located texture file is, keyed by the logical path that located it. */
      textures: { [key in string]: AssetTextureDescriptor };
    } | null>("plugin:visuals|get_model"),
  /**
   * Start browsing a world of visuals.
   *
   * Stores the intent rather than a listing: what the user chose is the world, and everything shown of it is derived
   * from that through the generic asset listing. A reload asks for this and derives the rest again.
   */
  openBrowse: (world: AssetWorldSpec) => __TAURI_INVOKE<null>("plugin:visuals|open_browse", { world }),
  /**
   * Select a visual and return what it contains, with every reference it declares resolved.
   *
   * Geometry is packed here and parked, so the `read_geometry` that follows serves the same parse rather than repeating
   * it. The bytes are not returned: a typed command cannot carry them, which is why they are read separately.
   *
   * Resolution happens once, for the whole dependency set, in this one call. That is what keeps a model with forty
   * textures from costing forty round trips, and it is why the outcomes travel with the description rather than being
   * asked for afterwards.
   */
  openModel: (source: VisualSource, world: AssetWorldSpec) =>
    __TAURI_INVOKE<SelectedVisualDescription>("plugin:visuals|open_model", { source, world }),
};
