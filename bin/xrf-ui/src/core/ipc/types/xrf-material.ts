// Auto-generated rust bindings. Do not edit it manually.

import { XrayAsset, XrayResolution } from "@/core/ipc/types/xrf-vfs";

/**
 * Which bump shader family a declaration selects, `STextureParams::ETBumpMode` without the two values that mean no
 * bump at all (`ETextureParams.h`).
 */
export enum EXrayBumpMode {
  USE = "use",
  PARALLAX = "parallax",
}

/** Every `EXrayBumpMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayBumpMode = `${EXrayBumpMode}`;

/**
 * What the renderer ends up drawing for a material, mirroring `Texture.cpp`.
 *
 * Ordered from best to worst so the outcome of a pair is the worse of its two inputs: a real bump over a dummy
 * companion is `Dummy`, and a dummy bump beside a companion that fell to the not-existing texture is `Missing`.
 */
export enum EXrayBumpOutcome {
  /** No usable declaration, so the flat shader variant is selected and no bump input is bound. */
  FLAT = "flat",
  /** Both inputs resolved to the files the declaration names. */
  BUMPED = "bumped",
  /**
   * The bump shader variant is selected and at least one input is the engine's flat dummy, because the declared name
   * contains `_bump` and its file is absent. The surface renders flat while paying the bump path, and the engine logs
   * `! Fallback to default bump map`.
   */
  DUMMY = "dummy",
  /**
   * At least one input is absent and has no dummy: its name lacks `_bump`, so `ed\ed_not_existing_texture` is bound
   * in its place, or nothing at all when even that is missing.
   */
  MISSING = "missing",
}

/** Every `EXrayBumpOutcome` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayBumpOutcome = `${EXrayBumpOutcome}`;

/** How a detail texture is applied, from the two texture param flags (`TextureDescrManager.cpp`). */
export enum EXrayDetailUsage {
  DIFFUSE = "diffuse",
  BUMP = "bump",
  DIFFUSE_AND_BUMP = "diffuseAndBump",
}

/** Every `EXrayDetailUsage` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type XrayDetailUsage = `${EXrayDetailUsage}`;

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

/** Every `kind` the `XrayMaterialDeclaration` union is told apart by, so a switch or a comparison names one. */
export enum EXrayMaterialDeclaration {
  /** No `.thm` sits beside the texture in any searched root. */
  NO_DESCRIPTOR = "noDescriptor",
  /** A `.thm` was located and could not be read as one. */
  UNREADABLE = "unreadable",
  /**
   * The descriptor's texture type is one `LoadTHM` skips whole (`TextureDescrManager.cpp`), so whatever its bump
   * chunk declares is never read. `declared_bump` is that chunk's used name, when it has one, so the panel can say
   * the declaration is fine and the type is not.
   */
  TYPE_DISQUALIFIED = "typeDisqualified",
  /** The descriptor carries no bump chunk at all. */
  NO_BUMP_CHUNK = "noBumpChunk",
  /** The bump chunk's mode is `none`, or the reserved value the engine clamps to it (`ETextureParams.cpp:77`). */
  DISABLED = "disabled",
  /** The mode asks for a bump and the name is empty, so `bump_exist()` is false and the flat shader is selected. */
  EMPTY_NAME = "emptyName",
  /** A bump the renderer will try to bind. */
  DECLARED = "declared",
}

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

/** Every `kind` the `XraySurfaceDeclaration` union is told apart by, so a switch or a comparison names one. */
export enum EXraySurfaceDeclaration {
  /** The surface names no shader at all, so there is nothing to look up. */
  UNDECLARED = "undeclared",
  /** No `shaders.xr` in any searched root, so nothing can be said about any surface of this model. */
  NO_LIBRARY = "noLibrary",
  /** A library was located and could not be read as one. */
  UNREADABLE = "unreadable",
  /** The library holds no blender of that name. */
  UNDEFINED = "undefined",
  /**
   * A blender whose class this crate does not derive a draw mode for, such as a particle or screen space class a
   * mesh has no business naming, or one a mod's renderer added.
   */
  UNMODELLED = "unmodelled",
  /** A blender whose class decides the surface from the knobs below. */
  DESCRIBED = "described",
}

/** What the shader library says about a surface, as the renderer would read it. */
export type XraySurfaceDeclaration =
  /** The surface names no shader at all, so there is nothing to look up. */
  | { kind: "undeclared" }
  /** No `shaders.xr` in any searched root, so nothing can be said about any surface of this model. */
  | { kind: "noLibrary" }
  /** A library was located and could not be read as one. */
  | { kind: "unreadable"; reason: string }
  /** The library holds no blender of that name. */
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
      /** The class's own alpha switch, or `None` for a class that writes none and is therefore always opaque. */
      isAlphaUsed: boolean | null;
      /** The authored `Alpha ref`, or `None` for a class that writes none. */
      alphaReference: number | null;
      /** `Strict sorting`, which every class writes and which pushes a model surface out of the deferred path. */
      isStrictSorting: boolean;
    };

/** How the renderer draws one surface, resolved from the shader name it declares and the textures it dresses with. */
export type XraySurfaceDescriptor = {
  /** The `shaders.xr` the answer was read from, or `None` when no root holds one. */
  library: XrayAsset | null;
  declaration: XraySurfaceDeclaration;
  /**
   * What to draw. Always answerable: a surface nothing could be read for is drawn the way the engine draws one whose
   * shader it could not resolve, which is opaque.
   */
  draw: XraySurfaceDraw;
  /**
   * The detail texture modulating its diffuse, `None` for a class the engine never details or a base texture whose
   * descriptor associates none.
   */
  detail: XraySurfaceDetail | null;
};

/** The detail texture a surface modulates its diffuse with, and how densely it is laid over it. */
export type XraySurfaceDetail = {
  /** Detail texture reference, engine-style, without extension. */
  reference: string;
  /** Times it repeats across the surface's base coordinate, `dt_params.xyz` (`TextureDescrManager.cpp`). */
  scale: number | null;
};

/** Every `kind` the `XraySurfaceDraw` union is told apart by, so a switch or a comparison names one. */
export enum EXraySurfaceDraw {
  /** Alpha is not read: whatever the texture carries in its fourth channel is ignored, and every texel is drawn. */
  OPAQUE = "opaque",
  /** Texels below the reference are killed and the rest are drawn opaque, in the g-buffer pass. */
  ALPHA_TESTED = "alphaTested",
  /** Drawn in a forward pass, source alpha over inverse source alpha, testing against the authored reference. */
  BLENDED = "blended",
  /** Added to what is behind it, which is how a glow lights the air rather than covering it. */
  ADDED = "added",
  /**
   * Multiplied into what is behind it, which is how a decal darkens the surface it is laid on rather than replacing
   * it. `is_doubled` is `MUL_2X`, whose destination factor is the source colour rather than zero.
   */
  MULTIPLIED = "multiplied",
}

/** How the renderer draws a surface once its blender is compiled. */
export type XraySurfaceDraw =
  /** Alpha is not read: whatever the texture carries in its fourth channel is ignored, and every texel is drawn. */
  | { kind: "opaque" }
  /** Texels below the reference are killed and the rest are drawn opaque, in the g-buffer pass. */
  | { kind: "alphaTested"; reference: number }
  /** Drawn in a forward pass, source alpha over inverse source alpha, testing against the authored reference. */
  | { kind: "blended"; reference: number }
  /** Added to what is behind it, which is how a glow lights the air rather than covering it. */
  | { kind: "added"; reference: number }
  /**
   * Multiplied into what is behind it, which is how a decal darkens the surface it is laid on rather than replacing
   * it. `is_doubled` is `MUL_2X`, whose destination factor is the source colour rather than zero.
   */
  | { kind: "multiplied"; isDoubled: boolean };
