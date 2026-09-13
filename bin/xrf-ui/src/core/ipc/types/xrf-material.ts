// Auto-generated rust bindings. Do not edit it manually.

import { XrayAsset, XrayResolution } from "@/core/ipc/types/xrf-vfs";

/**
 * Which bump shader family a declaration selects, `STextureParams::ETBumpMode` without the two values that mean no
 * bump at all (`ETextureParams.h`).
 *
 * Parallax changes the pixel shader only: `uber_deffer.cpp` compiles `_steep` for it in HQ mode and the same
 * `_bump` variant as [`Self::Use`] otherwise. The inputs bound are the same pair either way.
 */
export type XrayBumpMode = "use" | "parallax";

/**
 * What the renderer ends up drawing for a material, mirroring `Texture.cpp`.
 *
 * Ordered from best to worst so the outcome of a pair is the worse of its two inputs: a real bump over a dummy
 * companion is `Dummy`, and a dummy bump beside a companion that fell to the not-existing texture is `Missing`.
 */
export type XrayBumpOutcome =
  /** No usable declaration, so the flat shader variant is selected and no bump input is bound. */
  | "flat"
  /** Both inputs resolved to the files the declaration names. */
  | "bumped"
  /**
   * The bump shader variant is selected and at least one input is the engine's flat dummy, because the declared name
   * contains `_bump` and its file is absent. The surface renders flat while paying the bump path, and the engine logs
   * `! Fallback to default bump map`.
   */
  | "dummy"
  /**
   * At least one input is absent and has no dummy: its name lacks `_bump`, so `ed\ed_not_existing_texture` is bound
   * in its place, or nothing at all when even that is missing.
   */
  | "missing";

/**
 * How a detail texture is applied, from the two texture param flags (`TextureDescrManager.cpp`).
 *
 * A bump detail brings its own bump and bump# pair, looked up through the detail texture's own descriptor
 * (`uber_deffer.cpp`); that pair is not resolved here.
 */
export type XrayDetailUsage = "diffuse" | "bump" | "diffuseAndBump";

/**
 * A live bump declaration and both inputs it binds.
 *
 * Two inputs rather than one, because the companion is a separate file the engine derives by appending `#`
 * (`uber_deffer.cpp`) and it can be absent while the bump exists; that is the case the pair exists to show.
 */
export type XrayMaterialBump = {
  mode: XrayBumpMode;
  /**
   * Authoring data only. The renderer never reads it: parallax depth is the `r2_parallax_h` console variable, and
   * `bump_virtual_height` appears in `ETextureParams.cpp` load, save and the editor grid and nowhere else.
   */
  virtualHeight: number | null;
  /** `normal.gloss`, the texture the declaration names. */
  bump: XrayMaterialBumpInput;
  /** `normal_error.height`, the declared name with `#` appended. */
  companion: XrayMaterialBumpInput;
};

/** One of the two textures a bump declaration makes the renderer bind, and what binding it came to. */
export type XrayMaterialBumpInput = {
  /** The engine path the renderer asks for, verbatim. */
  reference: string;
  resolution: XrayResolution;
};

/**
 * What a texture's descriptor says about its bump, as the engine reads it.
 *
 * Every variant but [`Self::Declared`] renders flat, and they are six different things an author did. Collapsing them
 * into "no bump" is what leaves a modder opening a hex editor: a descriptor that looks complete and is skipped for
 * its type, and a descriptor that is simply absent, are the same surface in the viewport and opposite fixes.
 */
export type XrayMaterialDeclaration =
  /** No `.thm` sits beside the texture in any searched root. */
  | { kind: "noDescriptor" }
  /** A `.thm` was located and could not be read as one. */
  | { kind: "unreadable"; reason: string }
  /**
   * The descriptor's texture type is one `LoadTHM` skips whole (`TextureDescrManager.cpp`), so whatever its bump
   * chunk declares is never read. `declared_bump` is that chunk's used name, when it has one, so the panel can say
   * the declaration is fine and the type is not.
   */
  | { kind: "typeDisqualified"; textureType: number; label: string; declaredBump: string | null }
  /** The descriptor carries no bump chunk at all. */
  | { kind: "noBumpChunk" }
  /** The bump chunk's mode is `none`, or the reserved value the engine clamps to it (`ETextureParams.cpp:77`). */
  | { kind: "disabled"; mode: number }
  /** The mode asks for a bump and the name is empty, so `bump_exist()` is false and the flat shader is selected. */
  | { kind: "emptyName"; mode: XrayBumpMode }
  /** A bump the renderer will try to bind. */
  | { kind: "declared"; mode: XrayBumpMode; name: string };

/** The material the renderer builds for one texture, resolved. */
export type XrayMaterialDescriptor = {
  /** The `.thm` the declaration was read from, or `None` when no root holds one. */
  descriptor: XrayAsset | null;
  declaration: XrayMaterialDeclaration;
  /** The bound pair, present exactly when the declaration is [`XrayMaterialDeclaration::Declared`]. */
  bump: XrayMaterialBump | null;
  outcome: XrayBumpOutcome;
  /** The detail association the descriptor names, when the type gate lets the engine read it and it names one. */
  detail: XrayMaterialDetail | null;
};

/** The detail texture a descriptor names, and whether the engine applies it. */
export type XrayMaterialDetail = {
  /** Detail texture path without extension, engine-style, verbatim from the chunk. */
  name: string;
  scale: number | null;
  /**
   * `None` when the name is authored but neither detail flag is set, which the engine treats as no association
   * (`TextureDescrManager.cpp`). Reported rather than dropped, because dead authoring is a thing to fix.
   */
  usage: XrayDetailUsage | null;
};

/** What the shader library says about a surface, as the renderer would read it. */
export type XraySurfaceDeclaration =
  /** No `shaders.xr` in any searched root, so nothing can be said about any surface of this model. */
  | { kind: "noLibrary" }
  /** A library was located and could not be read as one. */
  | { kind: "unreadable"; reason: string }
  /**
   * The library holds no blender of that name.
   *
   * What the engine reports as `! Shader '%s' not found in library` before falling back to the default shader
   * (`Layers/xrRender/ResourceManager.cpp:40`), so the surface still draws - opaque, and not as authored.
   */
  | { kind: "undefined" }
  /**
   * A blender whose class this crate does not derive a draw mode for, such as a particle or screen space class a
   * mesh has no business naming, or one a mod's renderer added.
   */
  | { kind: "unmodelled"; class: string }
  /** A blender whose class decides the surface from the knobs below. */
  | {
      kind: "described";
      /** The class tag, as `Blender_CLSID.h` spells it: `MODEL`, `MODELEbB`, `LM_AREF`. */
      class: string;
      /**
       * The class's own alpha switch, or `None` for a class that writes none and is therefore always opaque.
       *
       * The engine spells it differently per class - `Use alpha-channel` for `B_MODEL`, `Alpha-blend` for
       * `B_DEFAULT_AREF`, `Alpha-Blend` for `B_MODEL_EbB` - and the reader keeps those apart.
       */
      isAlphaUsed: boolean | null;
      /**
       * The authored `Alpha ref`, or `None` for a class that writes none.
       *
       * Not necessarily what the surface tests against: see [`crate::XraySurfaceDraw::AlphaTested`].
       */
      alphaReference: number | null;
      /** `Strict sorting`, which every class writes and which pushes a model surface out of the deferred path. */
      isStrictSorting: boolean;
    };

/**
 * How the renderer draws one surface, resolved from the shader name it declares.
 *
 * The counterpart of [`crate::XrayMaterialDescriptor`], which answers the same question for a texture from its `.thm`.
 * Between them they are what a surface is made of: the shader decides whether alpha is read and how, the descriptor
 * decides what is bound beside the diffuse.
 */
export type XraySurfaceDescriptor = {
  /** The `shaders.xr` the answer was read from, or `None` when no root holds one. */
  library: XrayAsset | null;
  declaration: XraySurfaceDeclaration;
  /**
   * What to draw. Always answerable: a surface nothing could be read for is drawn the way the engine draws one whose
   * shader it could not resolve, which is opaque.
   */
  draw: XraySurfaceDraw;
};

/**
 * How the renderer draws a surface once its blender is compiled: opaque, cut out, or blended.
 *
 * The three cases a viewer has to reproduce, and the only three a mesh surface reaches. Which one a blender comes to
 * is [`crate::XraySurfaceResolver`]'s answer; what each one means is here.
 *
 * Modelled for the deferred renderer, R2 and above, because that is what the game runs and what a preview is compared
 * against. R1 differs in one place and the descriptor carries the knobs to say so: there the switch alone selects an
 * alpha blended pass and the authored reference is the test, where the deferred path tests against a constant.
 */
export type XraySurfaceDraw =
  /** Alpha is not read: whatever the texture carries in its fourth channel is ignored, and every texel is drawn. */
  | { kind: "opaque" }
  /** Texels below the reference are killed and the rest are drawn opaque, in the g-buffer pass. */
  | { kind: "alphaTested"; reference: number }
  /**
   * Drawn in a forward pass, source alpha over inverse source alpha, testing against the authored reference.
   *
   * Reached when the author asked for something the g-buffer cannot hold - a partly transparent surface, or one it
   * wants sorted - so the surface leaves the deferred path entirely. Depth is tested and not written
   * (`Layers/xrRender/blenders/blender_deffer_model.cpp`), which is what lets one blended surface show through
   * another.
   */
  | { kind: "blended"; reference: number };
