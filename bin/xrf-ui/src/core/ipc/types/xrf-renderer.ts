// Auto-generated rust bindings. Do not edit it manually.

/** How the screen's ambient occlusion is searched. */
export enum ERenderAmbientOcclusionMethod {
  /** GTAO as XeGTAO computes it: everything the depth shows taken to reach infinitely behind it. */
  GTAO = "gtao",
  /** VBAO, visibility-bitmask ambient occlusion: every occluder a thickness deep, so light passes behind thin things. */
  VBAO = "vbao",
}

/** Every `ERenderAmbientOcclusionMethod` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAmbientOcclusionMethod = `${ERenderAmbientOcclusionMethod}`;

/** How hard the ambient occlusion searches: XeGTAO's presets, and VBAO's own. */
export enum ERenderAmbientOcclusionQuality {
  /** GTAO one direction, two steps each way; VBAO three steps. */
  LOW = "low",
  /** GTAO two directions, two steps; VBAO five steps. */
  MEDIUM = "medium",
  /** GTAO three directions, three steps; VBAO eight steps. `Base`'s choice. */
  HIGH = "high",
  /** GTAO six directions, three steps; VBAO two directions, eight steps. */
  ULTRA = "ultra",
}

/** Every `ERenderAmbientOcclusionQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAmbientOcclusionQuality = `${ERenderAmbientOcclusionQuality}`;

/**
 * Ambient occlusion from the depth of the frame at half resolution: it darkens the hemisphere and ambient light over
 * the baked hemisphere occlusion, as the engine's SSAO does. The radius, strength and quality are every method's.
 */
export type RenderAmbientOcclusionSettings = {
  isEnabled: boolean;
  /** GTAO's horizons, or VBAO's visibility bitmask, which `vbao` shapes. */
  method: RenderAmbientOcclusionMethod;
  /** Metres around a point that what stands there occludes it from. */
  radius: number | null;
  /** How dark the occlusion goes: one the method's own curve, zero none, two its square. */
  strength: number | null;
  quality: RenderAmbientOcclusionQuality;
  vbao: RenderAmbientOcclusionVbaoSettings;
};

/**
 * VBAO's strengths: how deep an occluder is taken to be, how much light bounces back off a
 * surface into its own creases, and over how many frames it is gathered.
 */
export type RenderAmbientOcclusionVbaoSettings = {
  /** Metres behind what the depth shows that it is taken to be solid: light passes behind anything thinner. */
  thickness: number | null;
  /** How much of the light a surface's colour bounces between the sides of its creases comes back, one all of it. */
  bounce: number | null;
  /** Frames each pixel's occlusion is averaged over at most; one gathers none, its noise then holding still. */
  accumulation: number;
};

/** How a viewport's finished frame has its edges smoothed. */
export enum ERenderAntialiasing {
  /** Every edge as drawn. */
  NONE = "none",
  /** One pass over the drawn frame's edges, softest and cheapest. */
  FXAA = "fxaa",
  /** Three passes over the drawn frame's edges, crisp and stable. */
  SMAA = "smaa",
  /** Temporal: every frame's samples jittered within the pixel and resolved with the frames before. */
  TAA = "taa",
  /** FSR 2: temporal as TAA, with each surface's motion, disocclusion, reactivity and thin-feature locks. */
  FSR2 = "fsr2",
}

/** Every `ERenderAntialiasing` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAntialiasing = `${ERenderAntialiasing}`;

/** What the weather lights a scene with now, its keyframes blended, as the passes bind it. */
export type RenderAppliedEnvironment = {
  /** The direction sunlight travels, in renderer space. */
  sunDirection: [number | null, number | null, number | null];
  /** The sun's colour, times its light scale. */
  sunColor: [number | null, number | null, number | null];
  /** The ambient as combine binds it: doubled, floored, times its light scale. */
  ambient: [number | null, number | null, number | null];
  /** The hemisphere as combine binds it. */
  hemisphere: [number | null, number | null, number | null];
  /** Distance fog, or none. */
  fog: RenderAppliedFog | null;
  /** `rain_density`, zero for a dry sky. */
  rainDensity: number | null;
  /** `trees_amplitude`, zero for trees standing still. */
  treeSway: number | null;
  /** `water_intensity`. */
  waterIntensity: number | null;
};

/** Distance fog as drawn. */
export type RenderAppliedFog = {
  color: [number | null, number | null, number | null];
  /** Metres to where it is total, or the far plane where that is nearer. */
  distance: number | null;
  /** How near the camera it starts, a share of its distance. */
  density: number | null;
};

/** The grass as planted. */
export type RenderAppliedGrass = {
  /** Metres around the camera it is planted to, in whole slots. */
  radius: number | null;
  /** How far apart a slot's candidates stand, held to the engine's bounds. */
  density: number | null;
  height: number | null;
  /** Tufts the lists hold at most. */
  tufts: number;
  /** Tufts the radius and density would plant: more than `tufts` where a GPU buffer cannot hold them all. */
  wanted: number;
};

/** The screen-space indirect light as gathered. */
export type RenderAppliedIndirectLight = {
  /** How much of the bounced light is added. */
  intensity: number | null;
  /** Whether VBAO's own search gathers it, rather than a search of its own. */
  isShared: boolean;
};

/** The screen-space reflections as traced. */
export type RenderAppliedReflections = {
  /** What a surface's gloss and Fresnel term are scaled by into its share of reflection. */
  intensity: number | null;
  quality: RenderReflectionQuality;
};

/** What a viewport's frames are drawn with, as the renderer resolved what it was asked: sent as it changes. */
export type RenderAppliedReport = {
  /** What smooths the finished frame: none where a target other than the frame is shown. */
  antialiasing: RenderAntialiasing;
  /** How much smaller the scene is drawn than the viewport, native for anything but a level. */
  renderScale: RenderScale;
  /** The sun's shadow, or none where no cascade is drawn. */
  shadows: RenderAppliedShadows | null;
  /** The screen's ambient occlusion, or none where it is off or the scene is unlit. */
  ambientOcclusion: RenderAmbientOcclusionQuality | null;
  /** The screen-space indirect light, or none where it is not gathered. */
  indirectLight: RenderAppliedIndirectLight | null;
  /** The screen-space reflections, or none where they are not traced. */
  reflections: RenderAppliedReflections | null;
  /** The local lights, or none where they are off. */
  lights: RenderLightsSettings | null;
  /** The grass planted, or none where none is. */
  grass: RenderAppliedGrass | null;
  /** Whether the level's water is drawn. */
  isWater: boolean;
  /** What the weather lights the scene with now, or none for an asset viewer's rig. */
  environment: RenderAppliedEnvironment | null;
  /**
   * The lens flare whose sprite the sky draws, its `suns.ltx` section: the sun or the moon; none where the sky shows
   * neither.
   */
  sun: string | null;
};

/** The sun's shadow as drawn. */
export type RenderAppliedShadows = {
  /** Each cascade's width in metres, nearest first, as many as are drawn. */
  cascades: Array<number | null>;
  /** Texels each cascade's map is across, held to what the device allows. */
  resolution: number;
  /** Texels the filter reaches each way. */
  filter: number;
};

/**
 * How an asset viewer lights what it shows, in place of a level's weather: one light from a direction and a uniform
 * ambient, each a colour scaled by an intensity.
 */
export type RenderAssetLighting = {
  /** Degrees above the horizon the light sits at, ninety directly overhead. */
  sunElevation: number | null;
  /** Degrees around the vertical, zero looking along renderer `+z`. */
  sunAzimuth: number | null;
  sunIntensity: number | null;
  /** The light's colour, each channel zero to one. */
  sunColor: [number | null, number | null, number | null];
  ambientIntensity: number | null;
  ambientColor: [number | null, number | null, number | null];
};

/**
 * How an asset viewer stages what it shows: its own light, the backdrop behind it, and how its surfaces are textured
 * where it checks their coordinates or they name no texture.
 */
export type RenderAssetPreview = {
  /** An asset viewer's light, in place of the weather's; none for a level. */
  lighting: RenderAssetLighting | null;
  /**
   * What shows where nothing was drawn and neither the sky nor the fog is, each channel zero to one; none for the
   * level viewer's own.
   */
  backdrop: [number | null, number | null, number | null] | null;
  /** The backdrop laid out as a checkerboard with a second colour, as behind a picture with alpha; none for a plain one. */
  backdropSquares: RenderBackdropSquares | null;
  /** Times a uv checker repeats over a surface's base coordinate, drawn in place of its textures; zero for none. */
  checker: number | null;
  /** The colour a surface naming no base texture is drawn, each channel zero to one; none for white. */
  plainColor: [number | null, number | null, number | null] | null;
};

/**
 * A checkerboard where nothing was drawn, as the one behind a picture with alpha: the backdrop and a second colour in
 * squares.
 */
export type RenderBackdropSquares = {
  /** The second colour, each channel zero to one. */
  color: [number | null, number | null, number | null];
  /** Side of one square, in device pixels of the viewport. */
  size: number | null;
};

/** The engine's bloom (`phase_bloom`): the bright part of the frame blurred over it, as the console sets it. */
export type RenderBloomSettings = {
  isEnabled: boolean;
  /** `r2_ls_bloom_threshold`: what the summed brightness loses before it scales the blur. */
  threshold: number | null;
  /** `r2_ls_bloom_kernel_g`: the blur's radius, in texels of its 256-square target. */
  radius: number | null;
  /** `r2_ls_bloom_kernel_scale`: how strong the blur is. */
  strength: number | null;
};

/** An opaque colour as CSS states it, eight bits a channel in sRGB. */
export type RenderColor = {
  r: number;
  g: number;
  b: number;
};

/** Whether the sun's light is shadowed by what the frame's depth shows standing between a surface and the sun. */
export enum ERenderContactShadowMode {
  /** The engine's own: the sun's cascades alone. */
  ENGINE = "engine",
  /** Contact shadows: each pixel's ray towards the sun marched over the frame's depth, under the cascades. */
  ENHANCED = "enhanced",
}

/** Every `ERenderContactShadowMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderContactShadowMode = `${ERenderContactShadowMode}`;

/**
 * Contact shadows: the small shadows the sun's cascades and the lights' maps are too coarse to cast, or the lights
 * without a map do not cast at all, found in the frame's depth.
 */
export type RenderContactShadowSettings = {
  mode: RenderContactShadowMode;
  /** Metres each pixel's ray reaches towards the sun, or towards a light as far as the light at most. */
  length: number | null;
  /** How much of the light what the ray meets takes away: one all of it. */
  intensity: number | null;
  /** Metres behind what the depth shows that it is taken to be solid, near the camera; it grows with distance. */
  thickness: number | null;
  /** Depth reads along each ray. */
  steps: number;
  /** Local lights each pixel marches towards at most, the strongest there: none for the sun's alone. */
  lights: number;
};

/** Which picture a viewport shows: its finished frame, or one of the targets the frame was built from. */
export enum ERenderDebugView {
  /** The finished frame. */
  FINAL = "final",
  /** The G-buffer's albedo, raw. */
  ALBEDO = "albedo",
  /** The gloss the albedo target carries in its alpha. */
  GLOSS = "gloss",
  /** The view space normal, remapped to colour. */
  NORMAL = "normal",
  /** The baked hemisphere occlusion. */
  HEMI = "hemi",
  /** The baked sun occlusion. */
  SUN = "sun",
  /** The lighting model slice, `(class + 0.5) / 4`. */
  MATERIAL = "material",
  /** View distance, logarithmic: near dark, far light. */
  DEPTH = "depth",
  /** What the sun and the lights accumulated. */
  LIGHT = "light",
  /** The screen's occlusion, white where it is off. */
  AMBIENT_OCCLUSION = "ambientOcclusion",
  /** How far each surface moved on the screen since the last frame: red across, green down, grey still. */
  MOTION = "motion",
  /** The indirect light as it would light a white surface, black where it is not gathered. */
  INDIRECT_LIGHT = "indirectLight",
  /**
   * The screen-space reflections as traced: what a ray met in its colour, deep blue where it met nothing and the cube
   * stands, black where nothing is traced.
   */
  REFLECTIONS = "reflections",
}

/** Every `ERenderDebugView` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDebugView = `${ERenderDebugView}`;

/**
 * The enhanced water's strengths: what lies under it refracted, clouded with depth, the scene reflected,
 * bordered softly.
 */
export type RenderEnhancedWaterSettings = {
  /** How far its waves move what lies under it. */
  refraction: number | null;
  /** How deep it clears before it clouds into its colour. */
  turbidity: number | null;
  /** Metres of depth over which its edge fades into what lies under it. */
  softBorder: number | null;
  /** How much of its reflection it shows by the fresnel; none draws no reflection. */
  reflectivity: number | null;
  /** How far its reflection is blurred. */
  reflectionBlur: number | null;
  /** How much noise mixes the clear reflection into the blurred one. */
  blurNoise: number | null;
  /** How bright the sun's highlight on it is. */
  specular: number | null;
  /** How bright the light it gathers onto its bottom is. */
  caustics: number | null;
  /** How high its waves stand in its parallax; none draws it flat. */
  parallaxHeight: number | null;
  /** How strongly rain ripples it; none skips them. */
  ripples: number | null;
  /** What every scroll of its maps is multiplied by, one as designed; none stills it. */
  flow: number | null;
  /**
   * How much of its pace it keeps in still air, one as designed; none stills it while no
   * wind blows.
   */
  calmFlow: number | null;
  /**
   * How far it breaks its maps' repeat: its second layer tiled apart from the first, a broad layer faded in with
   * distance, and its colour's read bent and mixed with a second; none draws a single repeat.
   */
  variation: number | null;
};

/**
 * The engine's exposure (`r2_tonemap`): the frame's average luminance measured every frame, and the scale the tonemap
 * multiplies by moved towards `middle_gray / luminance` at the adaptation's rate.
 */
export type RenderExposureSettings = {
  /** Off, the tonemap multiplies by one, the engine's answer at noon. */
  isEnabled: boolean;
  /** `r2_tonemap_amount`: how far from no adaptation towards the whole of it. */
  amount: number | null;
  /** `r2_tonemap_middlegray`: the luminance the frame is brought towards. */
  middleGray: number | null;
  /** `r2_tonemap_lowlum`: what the luminance is floored at, so a black frame is not brightened without end. */
  lowLuminance: number | null;
  /** `r2_tonemap_adaptation`: how fast the scale follows the frame. */
  adaptation: number | null;
};

/** How trees and grass move in the wind. */
export enum ERenderFoliageMode {
  /**
   * The engine's own: each tree leant by a wind turning through the weather's `trees_*` keys, each waving tuft by
   * its wave.
   */
  ENGINE = "engine",
  /** Trunks swinging downwind and branches and grass carried by a flow field drifting with the weather's wind. */
  ENHANCED = "enhanced",
}

/** Every `ERenderFoliageMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderFoliageMode = `${ERenderFoliageMode}`;

/** How trees and grass move in the wind, and the enhanced motion's strengths. */
export type RenderFoliageSettings = {
  mode: RenderFoliageMode;
  /** The least share of the strongest wind the foliage moves at, from nothing to one. */
  minSpeed: number | null;
  /**
   * How fast the grass's flow field drifts, how far it tosses the tufts, how far the wind pushes them downwind, and
   * how much its gusts lift them.
   */
  grassSpeed: number | null;
  grassTurbulence: number | null;
  grassPush: number | null;
  grassWave: number | null;
  /** How fast the branches' flow field drifts, how fast the trunks swing, and how far. */
  treesSpeed: number | null;
  treesTrunk: number | null;
  treesBend: number | null;
  /** How much sunlight leaves and grass pass through from behind, and how much of the sun's colour that light keeps. */
  sssIntensity: number | null;
  sssColor: number | null;
};

/** Where the render thread's time goes a frame, mean milliseconds over a report's span, in the order it spends them. */
export type RenderFramePhases = {
  /** Moving the cameras and weathers on, and uploading the textures that arrived. */
  update: number | null;
  /** Waiting for the window's next image. */
  acquire: number | null;
  /** Taking in what the level's workers loaded. */
  load: number | null;
  /** Readying the frame: its uniforms, culling arguments, lights, particles and water. */
  prepare: number | null;
  /** Recording the level's passes. */
  record: number | null;
  /** Recording the window's own pass, which every viewport's picture is drawn into. */
  compose: number | null;
  /** Encoding what was recorded into the graphics API's commands, which wgpu leaves until the encoder is finished. */
  encode: number | null;
  /** Handing the encoded frame to the queue. */
  submit: number | null;
  /** Presenting it. */
  present: number | null;
};

/** How often a viewport's frames are drawn and how they are presented. */
export type RenderFrameRate = {
  /**
   * Frames a second drawn at most, or none for no cap: presented at the display's refresh while `is_vsync`, and as
   * fast as a frame is drawn otherwise.
   */
  limit: number | null;
  /** Whether a frame waits for a refresh of the display to be presented, which a faster limit stops at. */
  isVsync: boolean;
};

/** What a viewport's recent frames cost, reported a few times a second while it draws. */
export type RenderFrameReport = {
  /** Frames presented a second over the reported span. */
  framesPerSecond: number | null;
  /** Mean milliseconds between presented frames. */
  frameTime: number | null;
  /** Longest milliseconds between two presented frames. */
  frameTimeMax: number | null;
  /** Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting. */
  cpuTime: number | null;
  /** Where the render thread's time goes a frame, waiting for the window's image and presenting included. */
  phases: RenderFramePhases;
  /** Drawn width, in device pixels. */
  width: number;
  /** Drawn height, in device pixels. */
  height: number;
  /** The scene's width as rendered, smaller than the drawn one where it is upscaled. */
  renderWidth: number;
  /** And its height. */
  renderHeight: number;
  /** The graphics API drawn with. */
  backend: string;
  /** The GPU drawn on. */
  adapter: string;
  /** Whether its passes were timed on the GPU over the span. */
  isGpuTimed: boolean;
  /** What each pass cost on the GPU, in frame order; none while untimed. */
  passes: Array<RenderPassCost>;
  /** The level's static draws' pools and cull, empty without a level. */
  staticDraws: RenderStaticReport;
  /** The level's local lights, empty without a level. */
  lights: RenderLightsReport;
  /** The level's particle systems, empty without a level. */
  particles: RenderParticlesReport;
  /** Milliseconds the last sector taken in took to put into the scene, on the render thread. */
  sectorTime: number | null;
  /** What the renderer holds on the GPU. */
  memory: RenderMemoryReport;
  /** What the frame graph made of its latest frame, none before its first. */
  graph: RenderGraphReport | null;
};

/** One pass a viewport's frame ran, as the frame graph placed it. */
export type RenderGraphPass = {
  name: string;
  kind: RenderGraphPassKind;
  /** The encode group it is recorded in. */
  group: string;
  /** The render pass it draws in, which the raster passes beside it may share. */
  renderPass: number | null;
};

/**
 * What kind of pass a frame graph ran: a raster or compute pass, or an encoder pass recording copies and readbacks; a
 * bridge is an encoder pass the graph cannot check.
 */
export enum ERenderGraphPassKind {
  RASTER = "raster",
  COMPUTE = "compute",
  ENCODER = "encoder",
  BRIDGE = "bridge",
}

/** Every `ERenderGraphPassKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderGraphPassKind = `${ERenderGraphPassKind}`;

/**
 * What the frame graph made of a frame a viewport drew: its passes and its window's, in the order they ran, the
 * passes culled, the encode groups, the render passes, and the transients the whole frame made with the pooled
 * resources they share.
 */
export type RenderGraphReport = {
  passes: Array<RenderGraphPass>;
  culled: Array<string>;
  groups: Array<string>;
  /** Render passes the whole frame began. */
  renderPassCount: number;
  transients: Array<RenderGraphTransient>;
  /** Bytes the transients would take apart, and the pooled textures and buffers they share and their bytes. */
  transientBytes: number;
  pooledCount: number;
  pooledBytes: number;
};

/**
 * Which of the frame graph's mechanisms every frame compiles with, each to be turned off alone, or all for serial mode,
 * to bisect a difference in a capture.
 */
export type RenderGraphSettings = {
  /** Drops passes whose effects nothing reads. */
  isCulling: boolean;
  /** Lets transients whose lifetimes do not overlap share one texture or buffer. */
  isPooling: boolean;
  /** Draws consecutive raster passes into the same attachments in one render pass. */
  isMerging: boolean;
  /** Records each encode group into an encoder of its own, in parallel; off, the whole frame is one. */
  isGrouping: boolean;
};

/**
 * One resource a frame made for itself, as the frame graph placed it: its label, the pooled texture or buffer of its
 * kind it shares, the passes it lives between, and its bytes.
 */
export type RenderGraphTransient = {
  label: string;
  isTexture: boolean;
  ordinal: number;
  first: number;
  last: number;
  bytes: number;
};

/**
 * The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
 * G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
 */
export type RenderGrassSettings = {
  isEnabled: boolean;
  /** How far apart a slot's candidates stand, from 0.1 (the densest) to 0.99 (the sparsest): `r__detail_density`. */
  density: number | null;
  /** Whole metres around the camera grass is planted to: `r__detail_radius`. */
  radius: number | null;
  /** What every planted tuft is scaled by: `r__detail_height`. */
  height: number | null;
  /** How the trees and the grass move in the wind. */
  foliage: RenderFoliageSettings;
};

/**
 * Anomaly's `img_corrections`, which `combine_2` applies to the finished frame: `r__exposure`, `r__gamma`,
 * `r__saturation` and `r__color_grading`.
 */
export type RenderImageCorrections = {
  /** What the frame is multiplied by. */
  exposure: number | null;
  /** The power the frame is raised to, inverted. */
  gamma: number | null;
  /** How far from grey towards the frame's own colour: one leaves it. */
  saturation: number | null;
  /** The colour the mid tones are graded towards; black grades nothing. */
  grading: [number | null, number | null, number | null];
};

/** Whether the light surfaces bounce onto each other is gathered from what the frame shows. */
export enum ERenderIndirectLightMode {
  /** The engine's own: the hemisphere and the ambient alone stand for every bounce. */
  ENGINE = "engine",
  /** Screen-space indirect light: the sunlight and lamplight on what the frame shows, bounced once onto what faces it. */
  ENHANCED = "enhanced",
}

/** Every `ERenderIndirectLightMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderIndirectLightMode = `${ERenderIndirectLightMode}`;

/**
 * Screen-space indirect light: what the sun and the lamps light on screen, bounced once onto the surfaces facing it,
 * searched as VBAO searches and with it where VBAO occludes.
 */
export type RenderIndirectLightSettings = {
  mode: RenderIndirectLightMode;
  /** How much of the bounced light is added: one all of it. */
  intensity: number | null;
  /** Metres around a point it is gathered from; the occlusion's radius where that is further. */
  radius: number | null;
};

/** Every `kind` the `RenderLevelHit` union is told apart by, so a switch or a comparison names one. */
export enum ERenderLevelHit {
  /** A surface the level compiled. */
  SURFACE = "surface",
  /** An object the level's spawn places. */
  SPAWN = "spawn",
}

/** What of a level is drawn under a point of a viewport, and where the ray from the eye met it, in renderer space. */
export type RenderLevelHit =
  /** A surface the level compiled. */
  | {
      kind: "surface";
      sector: number;
      /** The shader table entry drawing it. */
      shaderId: number;
      /** The sector's instanced mesh it is one place of, or none for its baked geometry. */
      mesh: number | null;
      /** Which place of the mesh it is, or none for the baked geometry. */
      place: number | null;
      /** Whether it is a clump of trees drawn as its impostor. */
      isImpostor: boolean;
      point: [number | null, number | null, number | null];
    }
  /** An object the level's spawn places. */
  | {
      kind: "spawn";
      /** Its index among the level's spawned objects. */
      object: number;
      point: [number | null, number | null, number | null];
    };

/**
 * How a game's console scales the weather's light (`r2_sun_lumscale`, `r2_sun_lumscale_hemi`,
 * `r2_sun_lumscale_amb`): the sun's colour, and the hemisphere and the ambient the deferred frame is combined with.
 */
export type RenderLightScales = {
  sun: number | null;
  hemi: number | null;
  ambient: number | null;
};

/** How a shadowed local light's map is compared. */
export enum ERenderLightShadowFilter {
  /** `shadow_hw`: four bilinear comparisons 0.6 of a texel off the point, at `r2_ls_depth_bias` -0.0003, as vanilla. */
  ENGINE = "engine",
  /** Anomaly's `shadow_pcss`: a blocker search, then a penumbra of twelve comparisons, at its -0.001 bias. */
  SOFT = "soft",
}

/** Every `ERenderLightShadowFilter` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderLightShadowFilter = `${ERenderLightShadowFilter}`;

/** What a level's local lights came to on the last frame counted. */
export type RenderLightsReport = {
  /** Lights standing in view and lit. */
  inView: number;
  /** Of them, lights lit with their shadows. */
  shadowed: number;
  /** Lights in view past the most a frame holds, the farthest. */
  excess: number;
  /** Texels of the shadow atlas held, of its whole. */
  atlas: RenderPoolUse;
  /** Clusters of the view more lights reached than one holds, and the lights they left out. */
  fullClusters: number;
  dropped: number;
};

/** The level's local lights: binned into clusters of the view, and accumulated after the sun in one pass. */
export type RenderLightsSettings = {
  isEnabled: boolean;
  /** Whether the level file's own lights are drawn too, which the engine does only with `r2_allow_r1_lights`. */
  isLevelLights: boolean;
  /** Whether a light the engine shadows casts its shadows. */
  isShadowed: boolean;
  shadowFilter: RenderLightShadowFilter;
};

/**
 * How long a viewport's level had been opening when each part of it finished, each noted the first frame it is seen
 * finished; none for a part not finished yet.
 */
export type RenderLoadDurations = {
  /** Every sector taken in or failed. */
  sectors: number | null;
  /** Every spawned object's model in the scene. */
  spawn: number | null;
  /** The grass read. */
  grass: number | null;
  /** The local lights read. */
  lights: number | null;
  /** The particle systems read. */
  particles: number | null;
  /** Everything resident, every texture settled last: the whole load. */
  ready: number | null;
};

/**
 * Something of a level that could not be read, by what names it, and why: a sector by its index, a spawned model by
 * its visual's name.
 */
export type RenderLoadFailure = {
  name: string;
  reason: string;
};

/**
 * How far a viewport's level has been read and put on the GPU, sent as `RenderViewportEvent::Load` for the level drawn
 * and answered to a caller polling `describe_load` for the level last asked for.
 */
export type RenderLoadReport = {
  /** Sectors resident on the GPU. */
  sectors: number;
  /** Sectors the level has. */
  sectorsTotal: number;
  /** Bytes of the sectors resident, packed. */
  bytes: number;
  /** Textures uploaded or given up on. */
  textures: number;
  /** Textures the level samples: its scene's, its lights' projectors and its particles'. */
  texturesTotal: number;
  /**
   * Whether everything is resident, so the level draws as it will: every sector, the spawn, grass, lights and
   * particles read, and every texture settled.
   */
  isReady: boolean;
  /** How long each part took to finish, timed from when the level began opening. */
  durations: RenderLoadDurations;
};

/**
 * What decides how much of a level's static geometry draws at a distance: the engine's screen area thresholds, each in
 * pixels of a 90 degree lens before `r__geometry_lod` scales them.
 */
export type RenderLodSettings = {
  /** Whether distant trees are drawn as their impostors. */
  isImpostors: boolean;
  /** `r__geometry_lod`: every screen area threshold scales with it. */
  geometryLod: number | null;
  /** `r_ssaLOD_A` and `r_ssaLOD_B`: a clump's impostor draws below the first, its trees above the second. */
  ssaA: number | null;
  ssaB: number | null;
  /** `r_ssaDISCARD`: an instanced place smaller on screen than this is not drawn. */
  ssaDiscard: number | null;
  /** `r_ssaGLOD_start` and `r_ssaGLOD_end`: a progressive mesh is whole above the first, coarsest below the second. */
  ssaGlodStart: number | null;
  ssaGlodEnd: number | null;
};

/** What the renderer holds on the GPU for what it draws. */
export type RenderMemoryReport = {
  /** Bytes of every texture uploaded, shared by every viewport. */
  textures: number;
  /** Bytes of this viewport's scene buffers: geometry, clusters, places and the rest that grow with what it shows. */
  scene: number;
};

/** Every `kind` the `RenderOverlay` union is told apart by, so a switch or a comparison names one. */
export enum ERenderOverlay {
  /**
   * Line segments in renderer space: three floats a vertex, two vertices a segment, and three floats of colour a
   * vertex.
   */
  LINES = "lines",
  /** A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting. */
  SUN = "sun",
  /**
   * Points in renderer space, three floats each, drawn as discs of one colour `size` device pixels across wherever
   * they stand.
   */
  POINTS = "points",
  /** Every skinned model's bones as segments, child to parent, following its pose. */
  SKELETON = "skeleton",
}

/** A helper drawn over a viewport's frame, unlit, as the raw colours it names. */
export type RenderOverlay =
  /**
   * Line segments in renderer space: three floats a vertex, two vertices a segment, and three floats of colour a
   * vertex.
   */
  | {
      kind: "lines";
      positions: Array<number | null>;
      colors: Array<number | null>;
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    }
  /** A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting. */
  | { kind: "sun"; color: [number | null, number | null, number | null]; size: number | null }
  /**
   * Points in renderer space, three floats each, drawn as discs of one colour `size` device pixels across wherever
   * they stand.
   */
  | {
      kind: "points";
      positions: Array<number | null>;
      color: [number | null, number | null, number | null];
      size: number | null;
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    }
  /** Every skinned model's bones as segments, child to parent, following its pose. */
  | {
      kind: "skeleton";
      color: [number | null, number | null, number | null];
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    };

/** What the page shows where it is transparent around its viewports: its colour, and the wash laid over it. */
export type RenderPageBackdrop = {
  color: RenderColor;
  wash: RenderPageWash | null;
};

/** A linear gradient over a box of the page, as CSS `linear-gradient(angle, from, to)` paints it over a colour. */
export type RenderPageWash = {
  /** The box it is painted over, in device pixels of the window's client area. */
  rect: RenderRect;
  /** Degrees clockwise from pointing up, as CSS states a gradient's angle. */
  angle: number | null;
  /** Its first and last colours, sRGB channels and alpha from 0 to 1. */
  from: [number | null, number | null, number | null, number | null];
  to: [number | null, number | null, number | null, number | null];
};

/** What a level's particle systems came to over the span reported. */
export type RenderParticlesReport = {
  /** Effects playing, each of a group's counted. */
  effects: number;
  /** Particles alive in them. */
  particles: number;
  /** Effects that took an update on the last frame counted, drawn or scheduled. */
  simulated: number;
  /** Effects drawn on the last frame counted. */
  drawn: number;
  /** Mean milliseconds a frame spent stepping them, on the render thread's workers. */
  simulationTime: number | null;
};

/** What one pass of a viewport's frames cost on the GPU. */
export type RenderPassCost = {
  /** The pass, as the frame names it. */
  name: string;
  /** Mean GPU milliseconds a frame over the report's span, a frame it did not run in counting nought. */
  gpuTime: number | null;
};

/** How much of a pool of records a frame used: entries held, and entries it has room for before it grows. */
export type RenderPoolUse = {
  used: number;
  capacity: number;
};

/** A rectangle of a window's client area, in device pixels from its top left corner. */
export type RenderRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

/** What a glossy surface reflects: the sky's cube alone, or what the frame shows where a ray finds it. */
export enum ERenderReflectionMode {
  /** The engine's own: the irradiance cube along the reflection, weighed by the surface's gloss. */
  ENGINE = "engine",
  /**
   * Screen-space reflections: each glossy pixel blended towards what its reflected ray meets over the frame's depth,
   * by its gloss and a Fresnel term, towards the cube where the ray meets nothing.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderReflectionMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderReflectionMode = `${ERenderReflectionMode}`;

/**
 * How hard the screen-space reflections trace: the size traced at, the steps a ray takes at most, how far behind a
 * surface a step may land and still have met it, and whether a step that lands further is halved back once.
 */
export enum ERenderReflectionQuality {
  /** Half size, 16 steps. */
  LOW = "low",
  /** Half size, 24 steps. */
  MEDIUM = "medium",
  /** Half size, 32 steps, refined. */
  HIGH = "high",
  /** The frame's own size, 64 steps, refined. */
  ULTRA = "ultra",
}

/** Every `ERenderReflectionQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderReflectionQuality = `${ERenderReflectionQuality}`;

/**
 * Screen-space reflections on the frame's surfaces: each glossy surface blended towards what its reflected ray meets,
 * or the sky's cube where it meets nothing, by its share of reflection.
 */
export type RenderReflectionSettings = {
  mode: RenderReflectionMode;
  /** What a surface's gloss and Fresnel term are scaled by into its share of reflection; the share is at most one. */
  intensity: number | null;
  /** Metres a ray is traced at most. */
  distance: number | null;
  quality: RenderReflectionQuality;
};

/**
 * How much smaller than the viewport the scene is drawn and then upscaled: FSR's quality modes, by the ratio of the
 * viewport's side to the drawing's.
 */
export enum ERenderScale {
  NATIVE = "native",
  /** 1.5: two thirds of each side. */
  QUALITY = "quality",
  /** 1.7. */
  BALANCED = "balanced",
  /** 2: half of each side. */
  PERFORMANCE = "performance",
}

/** Every `ERenderScale` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderScale = `${ERenderScale}`;

/**
 * What of a viewport's level is selected, and the colour it is marked in: outlined where it is drawn, and a spawned
 * object boxed too.
 */
export type RenderSelection = {
  target: RenderSelectionTarget;
  /** sRGB from 0 to 1. */
  color: [number | null, number | null, number | null];
};

/** Every `kind` the `RenderSelectionTarget` union is told apart by, so a switch or a comparison names one. */
export enum ERenderSelectionTarget {
  /** An object the level's spawn places, by its index among them. */
  SPAWN = "spawn",
  /**
   * A surface the level compiled: one place of a sector's instanced mesh, or the sector's baked geometry of one shader
   * table entry.
   */
  SURFACE = "surface",
}

/** One thing of a level a selection names, as a pick names it. */
export type RenderSelectionTarget =
  /** An object the level's spawn places, by its index among them. */
  | { kind: "spawn"; object: number }
  /**
   * A surface the level compiled: one place of a sector's instanced mesh, or the sector's baked geometry of one shader
   * table entry.
   */
  | { kind: "surface"; sector: number; shaderId: number; mesh: number | null; place: number | null };

/** What every viewport of the renderer draws with. */
export type RenderSettings = {
  frameRate: RenderFrameRate;
  /** Whether each pass of a frame is timed on the GPU, where the device writes timestamps between passes. */
  isGpuTimed: boolean;
  /** Which of the frame graph's mechanisms the frames compile with. */
  graph: RenderGraphSettings;
};

/**
 * The sun's shadow: cascades of maps, each a square of the level seen from the sun, drawn through the static draws
 * and sampled by the sun's light. The engine's are three, 20, 40 and 160 metres across, at 2048 texels
 * (`render_phase_sun.cpp`, `r2_smap_size`).
 */
export type RenderShadowSettings = {
  isEnabled: boolean;
  /** Each cascade's width in metres, nearest first; as many cascades as widths, at most four. */
  cascades: Array<number | null>;
  /** Texels each cascade's map is across. */
  resolution: number;
  /** Texels the filter reaches from the one sampled, each way: zero for one comparison, one for a three by three. */
  filter: number;
  /** How far a point is moved along its normal before it is compared, in texels of its cascade. */
  bias: number | null;
  /** Metres towards the sun past a cascade that its casters may stand. */
  reach: number | null;
  /** How far in from a cascade's edge, as a share of its width, the next cascade is mixed in. */
  blend: number | null;
  /** Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not. */
  isStaggered: boolean;
  /**
   * The contact shadows: the sun's under the cascades, drawn only while the cascades are, and the local lights',
   * drawn whether the cascades are or not.
   */
  contact: RenderContactShadowSettings;
};

/**
 * What of a scene a view shows, each feeding which passes its frame graph declares: the sky and what crosses it, the
 * fog, the sun's flares and shafts, the wall marks, the particles, and the surfaces that cut out or blend.
 */
export type RenderShowFlags = {
  /** Whether the weather's sky is drawn behind the level, rather than a plain backdrop. */
  isSkyVisible: boolean;
  /** Whether the weather's clouds cross the sky. */
  isClouded: boolean;
  /** Whether the distance fades into the sky's haze rather than into the sky itself. */
  isSkyHazed: boolean;
  /** Whether the weather's fog hides the distance. */
  isFogged: boolean;
  /**
   * Whether the sun's lens flares are drawn over the frame, `disable_lens_flare 0`; its sprite and gradient are drawn
   * either way.
   */
  isLensFlared: boolean;
  /**
   * Whether the sun's light shafts are drawn through its shadow, `r2_sun_shafts` (`r2_sunshafts_mode volumetric` on
   * Monolith) at its highest quality.
   */
  isSunShafted: boolean;
  /** Whether the level's wall marks are laid over its surfaces. */
  isWallmarked: boolean;
  /** Whether the level's particle systems play and draw. */
  isParticled: boolean;
  /** Whether surfaces cut out and blend as their shaders ask, or draw solid. */
  isAlphaVisible: boolean;
};

/** How full a level's static draws' pools are, and what the camera's cull kept and occlusion hid. */
export type RenderStaticReport = {
  /** Slots, a draw each. */
  slots: RenderPoolUse;
  /** Places static draws stand in. */
  places: RenderPoolUse;
  /** Rows the instance cull tests, a spawned model's place each. */
  rows: RenderPoolUse;
  /** Impostors of clumps of trees. */
  lods: RenderPoolUse;
  /** Clusters static draws are made of. */
  clusters: RenderPoolUse;
  /** Entries the camera's visible list held, of the room it has. */
  surfaceList: RenderPoolUse;
  /** Indirect draws the camera's static batches issue a frame. */
  commands: number;
  /** Clusters the camera kept, and their triangles. */
  keptClusters: number;
  keptTriangles: number;
  /** Clusters the frustum kept and the depth hid, and their triangles. */
  occludedClusters: number;
  occludedTriangles: number;
};

/** How the sun's light shafts are drawn, beside the view's switch for them. */
export type RenderSunShafts = {
  quality: RenderSunShaftsQuality;
  /** `r2_sunshafts_min`, from zero to a half: the floor the keyframes' density is lifted from; zero draws it as it is. */
  minimum: number | null;
};

/**
 * How finely the sun's light shafts step along a ray: the engines' `r2_sun_shafts` and `r2_sunshafts_quality` short
 * of off, which the view's switch is.
 */
export enum ERenderSunShaftsQuality {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
}

/** Every `ERenderSunShaftsQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderSunShaftsQuality = `${ERenderSunShaftsQuality}`;

/** What colour a surface's albedo is drawn with. */
export enum ERenderSurfaceColor {
  /** Its own textures. */
  TEXTURED = "textured",
  /** One grey for every surface, so the shapes and the light read alone. */
  CLAY = "clay",
  /** A tint of its shader table entry, which tells neighbouring surfaces apart. */
  SHADER = "shader",
}

/** Every `ERenderSurfaceColor` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderSurfaceColor = `${ERenderSurfaceColor}`;

/** What became of one texture reference a viewport's scene samples. */
export type RenderTextureReport = {
  reference: string;
  state: RenderTextureState;
};

/** Every `kind` the `RenderTextureState` union is told apart by, so a switch or a comparison names one. */
export enum ERenderTextureState {
  LOADING = "loading",
  LOADED = "loaded",
  /** Its reference resolved to no file. */
  MISSING = "missing",
  /** Its file could not be read or laid out, and why. */
  FAILED = "failed",
}

/** Where one texture a viewport's scene samples stands. */
export type RenderTextureState =
  | { kind: "loading" }
  | {
      kind: "loaded";
      width: number;
      height: number;
      levels: number;
      /** The file's own layout, as the textures explorer names it. */
      layout: string;
      /** Whether it was expanded to eight bits a channel rather than uploaded as stored. */
      isExpanded: boolean;
    }
  /** Its reference resolved to no file. */
  | { kind: "missing" }
  /** Its file could not be read or laid out, and why. */
  | { kind: "failed"; reason: string };

/** What the scene is drawn at, a share of the viewport upscaled to it, and how sharply the upscaled frame is finished. */
export type RenderUpscalingSettings = {
  scale: RenderScale;
  /** RCAS's sharpness while upscaled, from none to its most. */
  sharpening: number | null;
};

/** Each of the renderer's features as a view sets it, a full set a view: one struct a feature, owned by its module. */
export type RenderViewFeatures = {
  exposure: RenderExposureSettings;
  bloom: RenderBloomSettings;
  shadows: RenderShadowSettings;
  ambientOcclusion: RenderAmbientOcclusionSettings;
  /** The light the frame's surfaces bounce onto each other, searched as the ambient occlusion's settings describe. */
  indirectLight: RenderIndirectLightSettings;
  /** What glossy surfaces reflect: the sky's cube, or what the frame shows where a ray finds it. */
  reflections: RenderReflectionSettings;
  lights: RenderLightsSettings;
  water: RenderWaterSettings;
  grass: RenderGrassSettings;
  /** How finely the sun's shafts step, and Monolith's floor under their density. */
  sunShafts: RenderSunShafts;
  /** How much of the static geometry draws at a distance. */
  lod: RenderLodSettings;
  /** How the frame's edges are smoothed. */
  antialiasing: RenderAntialiasing;
  /** What the finished frame is corrected by. */
  corrections: RenderImageCorrections;
  /** How the game's console scales the sun, the hemisphere and the ambient. */
  lightScales: RenderLightScales;
  /** How far the baked hemisphere darkens the ambient: zero for not at all. */
  hemiStrength: number | null;
  /** Whether what the last frame's depth hides is left undrawn. */
  isOcclusionCulled: boolean;
};

/**
 * How a view shades what it shows: lit or as its albedo, filled or as its edges, what colours the surfaces, whether
 * bumps bend them, and which picture it shows.
 */
export type RenderViewMode = {
  /** Whether the scene is lit, else shown as its raw albedo. */
  isLit: boolean;
  /** Whether every static surface draws as its triangles' edges. */
  isWireframe: boolean;
  /** What colour surfaces' albedo is drawn with: their textures, clay, or their shader's tint. */
  surfaceColor: RenderSurfaceColor;
  /** Whether bump textures bend the normal. */
  isBumped: boolean;
  /** Which picture the viewport shows: its frame, or one of the targets the frame was built from. */
  debugView: RenderDebugView;
};

/**
 * What one viewport draws its scene with, split by who owns each part: what it shows, how it shades it, each
 * feature's settings, what its frame is drawn at, and an asset viewer's staging. What of the world plays is the
 * world's own.
 */
export type RenderViewOptions = {
  show: RenderShowFlags;
  mode: RenderViewMode;
  features: RenderViewFeatures;
  output: RenderViewOutput;
  asset: RenderAssetPreview;
};

/** What a view's frame is drawn at before it is put into its rectangle, and how it is upscaled. */
export type RenderViewOutput = {
  /** What the scene is drawn at, and how its upscaled frame is sharpened. */
  upscaling: RenderUpscalingSettings;
  /**
   * Device pixels the scene is drawn tall before its render scale, or `None` for the viewport's own; one taller than
   * the viewport draws at the viewport's.
   */
  renderHeight: number | null;
};

/** Every `kind` the `RenderViewportEvent` union is told apart by, so a switch or a comparison names one. */
export enum ERenderViewportEvent {
  /** What the recent frames cost. */
  FRAME = "frame",
  /** What its frames are drawn with, as the renderer resolved what it was asked, sent as it changes. */
  APPLIED = "applied",
  /** How far its scene has loaded, sent as it changes. */
  LOAD = "load",
  /** The renderer cannot draw this viewport, and why. */
  FAILURE = "failure",
}

/** What the renderer tells a viewport's page. */
export type RenderViewportEvent =
  /** What the recent frames cost. */
  | { kind: "frame"; report: RenderFrameReport }
  /** What its frames are drawn with, as the renderer resolved what it was asked, sent as it changes. */
  | { kind: "applied"; report: RenderAppliedReport }
  /** How far its scene has loaded, sent as it changes. */
  | { kind: "load"; report: RenderLoadReport }
  /** The renderer cannot draw this viewport, and why. */
  | { kind: "failure"; message: string };

/** One viewport of the renderer, numbered by the renderer as it is attached. */
export type RenderViewportId = number;

/** Where a viewport sits in its window and what the page shows around it. */
export type RenderViewportLayout = {
  /** The viewport element's rectangle, in device pixels of the window's client area. */
  rect: RenderRect;
  /** Device pixels per CSS pixel, the unit input coordinates are given in. */
  scale: number | null;
  /**
   * What the page shows where it is transparent, painted under the viewports: around them, and where one has not
   * followed a layout change yet.
   */
  backdrop: RenderPageBackdrop;
};

/** Which water a viewport draws. */
export enum ERenderWaterMode {
  /** The engine's own: `water.ps` and `waterd.ps`, reflecting the sky over the base. */
  ENGINE = "engine",
  /** The enhanced water: what lies under it refracted, clouded with depth, the scene reflected, bordered softly. */
  ENHANCED = "enhanced",
}

/** Every `ERenderWaterMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderWaterMode = `${ERenderWaterMode}`;

/**
 * The water (`water.vs`, `water.ps`, `waterd.ps`): rippled and reflecting the sky, blended over the depth behind it
 * and distorting it. The engine's own look by default: its constants are `shared/waterconfig.h`'s and `def_distort`.
 */
export type RenderWaterSettings = {
  /** Off, what lies under the water shows. */
  isEnabled: boolean;
  /** The engine's water, or the enhanced water, which `enhanced` shapes. */
  mode: RenderWaterMode;
  /** `r2_soft_water`: soft water fades by the depth behind it, darkens with it and lays foam in the shallows. */
  isSoft: boolean;
  /** Whether water writes the distortion it causes, moving what is seen through it. */
  isDistorted: boolean;
  /** How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`. */
  waveHeight: number | null;
  /** How fast they run: `W_POSITION_SHIFT_SPEED`. */
  waveSpeed: number | null;
  /** What the two normal layers' scroll is multiplied by, one as the engine scrolls them. */
  ripple: number | null;
  /** What the sky's reflection is multiplied by, one as the engine mixes it. */
  reflection: number | null;
  /**
   * How far the distortion target moves what is behind it, a share of the screen: `def_distort`, which moves what
   * the distorting particles write as well.
   */
  distortion: number | null;
  enhanced: RenderEnhancedWaterSettings;
};
