// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { AssetTextureDescriptor, SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { XrayAsset, XrayRoot, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { VisualDependencies, VisualDescription, VisualMotionBake } from "@/core/bindings/types/xrf-visual";

/** Commands */
export const visualsCommands = {
  /**
   * Stop browsing, leaving whatever visual is open on screen.
   *
   * The mounted sources stay: they belong to the shared asset roots, which outlives any one session and is what makes
   * browsing the same root again free.
   */
  closeBrowse: () => __TAURI_INVOKE<null>("plugin:visuals|close_browse"),
  /** Drop the selected visual and its packed geometry. */
  closeModel: () => __TAURI_INVOKE<null>("plugin:visuals|close_model"),
  /**
   * The roots the viewer was browsing, or null when it was showing one visual on its own.
   *
   * The rehydration probe for the tree, beside the one the selection already has: a reloaded frontend asks what is
   * being browsed and lists it again, so the panel comes back rather than emptying beside a model still open.
   */
  getBrowse: () =>
    __TAURI_INVOKE<{
      /**
       * Native asset address whose own X-Ray root and installation are searched first, when the read is centred on one.
       *
       * This is what finds a texture shipped beside a model rather than in the shared tree.
       */
      asset: string | null;
      /** Roots searched after the asset's own, in the order given. */
      roots: Array<XrayRoot>;
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
      /** The roots the selection was opened in, so a reloaded frontend asks for geometry the same way. */
      roots: XrayRoots;
      description: VisualDescription;
      dependencies: VisualDependencies;
      /** What each located texture file is, keyed by the logical path that located it. */
      textures: { [key in string]: AssetTextureDescriptor };
      /** What the renderer builds for each declared texture, keyed by the reference as the mesh declares it. */
      materials: { [key in string]: XrayMaterialDescriptor };
      /** A `textures.ltx` the searched roots hold, or `None`. */
      texturesLtx: XrayAsset | null;
    } | null>("plugin:visuals|get_model"),
  /**
   * Every motion the open visual can play, by name.
   *
   * Asked for rather than returned by `open_model`, because naming them means reading each animation file the visual
   * references - about fifty milliseconds each against a seventy millisecond open. The viewer already knows whether a
   * visual animates at all, from its references, so nothing needs this until something is about to play one.
   */
  listMotions: () => __TAURI_INVOKE<Array<string>>("plugin:visuals|list_motions"),
  /**
   * Start browsing roots of visuals.
   *
   * Stores the intent rather than a listing: what the user chose is the roots, and everything shown of it is derived
   * from that through the generic asset listing. A reload asks for this and derives the rest again.
   */
  openBrowse: (roots: XrayRoots) => __TAURI_INVOKE<null>("plugin:visuals|open_browse", { roots }),
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
  openModel: (source: VisualSource, roots: XrayRoots) =>
    __TAURI_INVOKE<SelectedVisualDescription>("plugin:visuals|open_model", { source, roots }),
  /**
   * Pose the open visual through one of its motions, and report what came out.
   *
   * Every frame is baked here and parked, so the `read_motion` that follows serves the same pose rather than composing
   * it again - the same split geometry uses, and for the same reason: a typed command cannot carry the bytes.
   *
   * Baked whole rather than sampled per frame because playback runs at thirty frames a second. A measured motion
   * averages 78 frames, which for a fifty bone skeleton is tens of kilobytes: cheaper once than eighty times.
   */
  openMotion: (name: string) => __TAURI_INVOKE<VisualMotionBake>("plugin:visuals|open_motion", { name }),
};
