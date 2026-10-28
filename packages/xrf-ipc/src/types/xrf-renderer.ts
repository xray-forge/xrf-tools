// Auto-generated rust bindings. Do not edit it manually.

/** Pixels a cubemap's face spans on a side at most. */
export const RENDER_CUBE_MOST_PIXELS: number = 2048;

/** Times over a cubemap's face is drawn at most. */
export const RENDER_CUBE_MOST_SUPERSAMPLING: number = 4;

/**
 * GPU milliseconds a frame gives the work drawn once in so many frames that may wait, unless the settings say
 * otherwise.
 */
export const RENDER_DEFAULT_PERIODIC_BUDGET: number = 0.5;

/** Map pixels a level map spans on a side at most. */
export const RENDER_MAP_MOST_PIXELS: number = 16384;

/** Times over a level map's tile is drawn at most. */
export const RENDER_MAP_MOST_SUPERSAMPLING: number = 4;

/** Map pixels a level map's tile spans on a side at most. */
export const RENDER_MAP_TILE: number = 2048;

/** Map pixels a level map's tile is drawn past each side, as far as the screen-space passes gather around a pixel. */
export const RENDER_MAP_TILE_MARGIN: number = 64;

/** Frames VBAO accumulates over at most. */
export const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: number = 32;

/** Local lights a pixel marches contact shadows towards at most: the lights' shader keeps their weights in an array. */
export const RENDER_MAX_CONTACT_SHADOW_LIGHTS: number = 8;

/** Cascades the sun's shadow can be cut into at most: the sun reads their texels from one `vec4`. */
export const RENDER_MAX_SHADOW_CASCADES: number = 4;

/** Texels the sun's shadow filter reaches each way at most: the widest the sun's pass has a pipeline for. */
export const RENDER_MAX_SHADOW_FILTER: number = 3;

/** GTAO's search: the slices it turns about a pixel and the steps it takes each way along each. */
export type RenderAmbientOcclusionGtaoSettings = {
  slices: number;
  steps: number;
};

/** How the screen's ambient occlusion is searched. */
export enum ERenderAmbientOcclusionMethod {
  /** GTAO as XeGTAO computes it: everything the depth shows taken to reach infinitely behind it. */
  GTAO = "gtao",
  /** VBAO, visibility-bitmask ambient occlusion: every occluder a thickness deep, so light passes behind thin things. */
  VBAO = "vbao",
  /**
   * RTAO: short rays from each point traced against the level's geometry by hardware ray query, so what is off screen
   * or hidden behind something nearer occludes too; VBAO searches in its place on a GPU that traces no rays.
   */
  RTAO = "rtao",
}

/** Every `ERenderAmbientOcclusionMethod` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderAmbientOcclusionMethod = `${ERenderAmbientOcclusionMethod}`;

/** RTAO's trace: the rays each point casts a frame. */
export type RenderAmbientOcclusionRtaoSettings = {
  rays: number;
};

/**
 * Ambient occlusion from the depth of the frame at half resolution: it darkens the hemisphere and ambient light over
 * the baked hemisphere occlusion, as the engine's SSAO does. The radius and strength are every method's; each method
 * has its own search.
 */
export type RenderAmbientOcclusionSettings = {
  isEnabled: boolean;
  /**
   * GTAO's horizons, VBAO's visibility bitmask, which `vbao` shapes, or RTAO's traced rays, accumulated and filtered
   * as VBAO's search is.
   */
  method: RenderAmbientOcclusionMethod;
  /** Metres around a point that what stands there occludes it from. */
  radius: number | null;
  /** How dark the occlusion goes: one the method's own curve, zero none, two its square. */
  strength: number | null;
  gtao: RenderAmbientOcclusionGtaoSettings;
  vbao: RenderAmbientOcclusionVbaoSettings;
  rtao: RenderAmbientOcclusionRtaoSettings;
};

/**
 * VBAO's search and strengths: the slices it turns about a pixel and the steps along each, how deep an occluder is
 * taken to be, how much light bounces back off a surface into its own creases, and over how many frames it is
 * gathered.
 */
export type RenderAmbientOcclusionVbaoSettings = {
  slices: number;
  steps: number;
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
  /**
   * FSR 3.1: AMD's temporal upscaler, temporal as TAA with each surface's motion, disocclusion, shading change,
   * reactivity and thin-feature locks.
   */
  FSR = "fsr",
  /** DLSS: NVIDIA's learned temporal upscaler, on an RTX GPU; FSR where the GPU has none. */
  DLSS = "dlss",
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

/** The enhanced fog as drawn. */
export type RenderAppliedHeightFog = {
  /** Metres either side of the world's zero height the height fog rises over. */
  height: number | null;
  /** How much thicker the distance fog grows where the height fog is whole. */
  density: number | null;
  /** How far it takes the sun's colour facing the sun. */
  sunColor: number | null;
  /** How readily the frame's bright parts are blurred into it, zero where nothing is scattered. */
  scattering: number | null;
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
  /** What scales how much of a surface's reflection a traced one replaces, and how much a puddle's coat reflects. */
  intensity: number | null;
  /** The size the rays are traced at, and the pyramid's cells a ray crosses at most. */
  traceSize: RenderTraceSize;
  steps: number;
};

/** What a viewport's frames are drawn with, as the renderer resolved what it was asked: sent as it changes. */
export type RenderAppliedReport = {
  /** What smooths the finished frame: none where a target other than the frame is shown. */
  antialiasing: RenderAntialiasing;
  /** Whether DLSS reconstructs rays as it upscales the frame. */
  isRayReconstructed: boolean;
  /** How much smaller the scene is drawn than the viewport, native for anything but a level. */
  renderScale: RenderScale;
  /** The sun's shadow, or none where no cascade is drawn. */
  shadows: RenderAppliedShadows | null;
  /** The screen's ambient occlusion, or none where it is off or the scene is unlit. */
  ambientOcclusion: RenderAmbientOcclusionSettings | null;
  /**
   * How the ambient occlusion is found, or none where it is not: RTAO asked for is VBAO's search on a GPU that traces
   * no rays.
   */
  ambientOcclusionMethod: RenderAmbientOcclusionMethod | null;
  /** The screen-space indirect light, or none where it is not gathered. */
  indirectLight: RenderAppliedIndirectLight | null;
  /** The screen-space reflections, or none where they are not traced. */
  reflections: RenderAppliedReflections | null;
  /** The enhanced height fog, or none where the engine's fog alone is drawn or none at all. */
  heightFog: RenderAppliedHeightFog | null;
  /** How hard the sky is debanded, or none where it is drawn as it is. */
  debanding: number | null;
  /** Which bloom is drawn, or none where the frame blooms nothing. */
  bloom: RenderBloomMode | null;
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
  /**
   * The backdrop laid out as a checkerboard with a second colour, as behind a picture with alpha; none for a plain
   * one.
   */
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

/** The graphics API the renderer draws with. */
export enum ERenderBackend {
  D3D12 = "d3d12",
  VULKAN = "vulkan",
  /** Apple's API: experimental, as no CI job builds or tests it. */
  METAL = "metal",
}

/** Every `ERenderBackend` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderBackend = `${ERenderBackend}`;

/** Whether the renderer can draw with a backend on this machine: the GPUs it can use, or why it cannot draw. */
export type RenderBackendAvailability = {
  backend: RenderBackend;
  /**
   * Every hardware GPU of the backend with the features the renderer needs, the one chosen automatically first; empty
   * if it cannot draw.
   */
  gpus: Array<RenderGpuDescription>;
  /** Why it cannot draw; none if it can. */
  problem: string | null;
  /** Whether `XRF_RENDER_BACKEND` names it, so the renderer starts on it instead of the settings' choice. */
  isForced: boolean;
};

/** Which bloom glows over the frame's bright parts: the engine's, as the look sets it, or the enhanced one. */
export enum ERenderBloomMode {
  /** The engine's own (`phase_bloom`), drawn as the look's bloom settings say. */
  ENGINE = "engine",
  /**
   * The enhanced bloom in its place: the finished frame's bright parts and self-lit surfaces blurred over six halving
   * sizes and back, tonemapped and screened over the frame.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderBloomMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderBloomMode = `${ERenderBloomMode}`;

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

/**
 * A run of consecutive frames asked of a viewport, drawn on a clock of its own a fixed step apart from a set time, so
 * two bursts from the same time draw the same scene however long their frames take; each read back whole as a
 * screenshot is, with its G-buffer's marks.
 *
 * Its first frame restarts what the view carries from frame to frame (the wind's drift, the temporal resolve's
 * history and jitter), so what it draws depends on its own frames alone. A reference burst draws each frame's time
 * `samples` times over, its clock held, at as many places of the resolve's jitter, unresolved and at the viewport's
 * own size: what a perfect temporal resolve converges to at that time, once averaged.
 */
export type RenderBurst = {
  /** The scene's seconds, since its level began opening, its first frame is drawn at. */
  at: number | null;
  /** Seconds between its frames. */
  step: number | null;
  /** Frames drawn before the first read back, for what builds over frames to settle from its restart. */
  leadIn: number;
  /** Frames read back, one after another. */
  frames: number;
  /** Samples each frame read back is drawn at for a reference; none for the frames as the view resolves them. */
  samples: number | null;
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
  /**
   * Whether the sun's are marched at the frame's size rather than half of it: at half, a temporal resolve's jitter
   * moves a thin blade on and off the one texel each two by two is marched from, which flickers.
   */
  isFullResolution: boolean;
  /** Local lights each pixel marches towards at most, the strongest there: none for the sun's alone. */
  lights: number;
};

/**
 * A cubemap asked of a viewport: the six faces around where its camera stands, world-aligned in the engine's order
 * `+x -x +y -y +z -z`, each `size` pixels square at exactly 90 degrees.
 */
export type RenderCubeCapture = {
  /** A face's pixels on a side. */
  size: number;
  /** Times over on each side a face is drawn before it is reduced to its pixels. */
  supersampling: number;
  /** Frames each face is drawn from where it stands before the one kept, for what builds over frames to settle. */
  settleFrames: number;
};

/** Whether the sky's gradients are smoothed where its colours fall into visible bands. */
export enum ERenderDebandingMode {
  /** The engine's own: the sky as drawn. */
  ENGINE = "engine",
  /**
   * Sky debanding: each sky pixel averaged with neighbours scattered around it wherever they differ from it by less
   * than a band's step.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderDebandingMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDebandingMode = `${ERenderDebandingMode}`;

/** Whether and how the sky's colour bands are smoothed. */
export type RenderDebandingSettings = {
  mode: RenderDebandingMode;
  /** Times a sky pixel is averaged with its neighbours, each time further out. */
  passes: number;
  /** Pixels around a sky pixel its neighbours are read from at most. */
  radius: number | null;
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
  /** The lighting model slice the G-buffer keeps (`material_slice`). */
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
  /** How each reflection ray ended, one colour a reason (`common/reflection_end.wgsl`); black where nothing is traced. */
  REFLECTION_ENDS = "reflectionEnds",
  /**
   * How far parallax moved each surface's texture: across in red and down in green about a half, out to the march's
   * reach, over full blue; a half in both where a surface takes parallax but stands past its reach; every other
   * surface a dim grey of its own brightness.
   */
  PARALLAX = "parallax",
  /**
   * Which surfaces are bumped: a bumped one's normal in its own tangent space as a normal map colours it, lavender
   * where flat, a terrain's by its four details' bumps; a surface without a bump red by its brightness.
   */
  BUMPS = "bumps",
  /**
   * The heights parallax reads: a terrain's details' weighed by its mask, an installation's own or derived from their
   * bumps, and a steep surface's, grey from black low to white high; every other surface a dim blue.
   */
  HEIGHTS = "heights",
  /**
   * The scene's luminance in stops about middle grey, one colour a stop: blues below, grey at middle grey, green to
   * red above, white six stops over and black six under.
   */
  LUMINANCE = "luminance",
  /** The albedo's luminance read as linear light: grey within the plausible 0.03 to 0.9, blue below, red above. */
  ALBEDO_RANGE = "albedoRange",
  /**
   * How many of the static scene's surfaces were drawn over each pixel, hidden or not: black none, then blue through
   * green and yellow to red at eight.
   */
  OVERDRAW = "overdraw",
  /**
   * Which of the sun's cascades shades each pixel, a tint a cascade from the nearest, darker where it shadows, grey
   * past them all; striped within the reach the grass's cards cast their shadows to.
   */
  CASCADES = "cascades",
  /**
   * How many lamps the cluster each pixel falls in lists: black none, then blue through green and yellow to red at
   * sixteen.
   */
  LAMPS_PER_CLUSTER = "lampsPerCluster",
  /**
   * The mip a surface's base texture is read at: green its own size, blue where it is magnified, yellow to red as
   * smaller mips are read; grey where a surface has no base texture.
   */
  SAMPLED_MIP = "sampledMip",
  /** The specular the sun and the lights accumulated, kept apart from their diffuse in the light target's alpha. */
  SPECULAR = "specular",
  /**
   * FSR's masks at each pixel's drawn texel: how reactive it is (moving apart from what it lands on, or a plant the
   * wind moved) in red, how disoccluded in green, how far its shading changed in blue.
   */
  FSR_MASKS = "fsrMasks",
  /** How far FSR trusts each pixel's history, from black for none to white; red where it starts afresh. */
  FSR_HISTORY_TRUST = "fsrHistoryTrust",
  /**
   * FSR's thin-feature locks: each pixel's lock in green, full where it holds the history against clamping; red
   * where a new lock was found this frame.
   */
  FSR_LOCKS = "fsrLocks",
  /** How strongly FSR finds each pixel's luma flickering back to an earlier frame's, which lets its history hold. */
  FSR_LUMA_INSTABILITY = "fsrLumaInstability",
  /**
   * How much of each pixel FSR takes from this frame rather than its history, from black for none to white for all,
   * as its square root so a frame's usual small share reads.
   */
  FSR_FRAME_SHARE = "fsrFrameShare",
}

/** Every `ERenderDebugView` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDebugView = `${ERenderDebugView}`;

/**
 * What turns the physical light's scene-referred frame into the display's colours, before it is encoded for the
 * display. The engine's light model always shows its frame by the engine's curve.
 */
export enum ERenderDisplayTransform {
  /**
   * The engine's curve, `x (1 + x / W²) / (1 + x)` a channel, white at 1.7, over the light encoded for the display as
   * the engine's values are: the authored levels shown as the engine shows them. Each channel clips on its own, so
   * bright colours walk to yellow and white.
   */
  ENGINE = "engine",
  /**
   * AgX (Troy Sobotka; Blender's default view since 4.0): a log encoding and a sigmoid over a rotated, inset gamut,
   * so highlights roll off towards white without their hues walking.
   */
  AGX = "agx",
  /**
   * Khronos PBR Neutral (2024): colours kept as they are up to a shoulder near white, then compressed and desaturated,
   * for materials read as authored; its toe, which squares the dark, is left out.
   */
  PBR_NEUTRAL = "pbrNeutral",
}

/** Every `ERenderDisplayTransform` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDisplayTransform = `${ERenderDisplayTransform}`;

/** Whether a viewport's GPU upscales with DLSS, or reconstructs rays with it, and where it does not, what it lacks. */
export enum ERenderDlssAvailability {
  AVAILABLE = "available",
  /** The build has no DLSS: it was built without the `dlss` feature. */
  UNBUILT = "unbuilt",
  /** The GPU or its driver has no DLSS, or none of the feature. */
  UNSUPPORTED = "unsupported",
}

/** Every `ERenderDlssAvailability` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDlssAvailability = `${ERenderDlssAvailability}`;

/** Which of DLSS's models upscales: NVIDIA's pick for each quality, or one for every quality. */
export enum ERenderDlssPreset {
  /**
   * NVIDIA's pick, which a driver update may change: K for DLAA, Quality and Balanced, M for Performance, L for Ultra
   * Performance.
   */
  DEFAULT = "default",
  /** Transformer, like K, with slightly less ghosting and more flicker. */
  J = "j",
  /** Transformer: the best image, at the highest cost. */
  K = "k",
  /** NVIDIA's model for Ultra Performance. */
  L = "l",
  /** NVIDIA's model for Performance. */
  M = "m",
}

/** Every `ERenderDlssPreset` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderDlssPreset = `${ERenderDlssPreset}`;

/** Which bloom the frame draws, and the enhanced bloom's strengths; the engine bloom's own come with the look. */
export type RenderEnhancedBloomSettings = {
  mode: RenderBloomMode;
  /** The power the finished frame is raised to before it blooms: the higher, the brighter a part must be to glow. */
  threshold: number | null;
  /** How bright the bloom is tonemapped. */
  exposure: number | null;
  /** How far apart each halving size's reads are spread, in its texels: the bloom's width. */
  blur: number | null;
  /** How saturated the bloom's colour is: one as it is, none grey. */
  vibrance: number | null;
  /** How much the sky blooms, against the rest of the frame. */
  sky: number | null;
};

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
  /** How much of its colour the reflection keeps, one all of it, none grey. */
  vibrance: number | null;
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
  /**
   * Stops the physical light's exposure lifts the frame over the level it meters it at, or lowers it where negative;
   * the engine's exposure has no such setting.
   */
  compensation: number | null;
};

/**
 * What fog closes the level in: the weather's distance fog alone, or that fog thickened low down and scattering the
 * frame's light.
 */
export enum ERenderFogMode {
  /** The engine's own: the weather's distance fog. */
  ENGINE = "engine",
  /**
   * The distance fog thickened below a height and tinted towards the sun there, with the bright parts of the frame
   * blurred into where it lies.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderFogMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderFogMode = `${ERenderFogMode}`;

/** How the fog is drawn beyond the weather's own keys, and the enhanced fog's strengths. */
export type RenderFogSettings = {
  mode: RenderFogMode;
  /** Metres either side of the world's zero height over which the height fog rises from none, above, to whole, below. */
  height: number | null;
  /** How much thicker the distance fog grows where the height fog is whole: one doubles it. */
  density: number | null;
  /** How far the height fog takes the sun's colour where the view faces the sun. */
  sunColor: number | null;
  /** How readily the bright parts of the frame are blurred into the fog: none blurs nothing. */
  scattering: number | null;
  /**
   * The fog's share past which fogged surfaces fade into the sky behind them, whole where the fog is: none fades them
   * by the fog's square from the first, as the engine does, which shows the sky through the middle distance.
   */
  skyFade: number | null;
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

/** Where the render thread's time goes a frame, mean milliseconds over a report's span. */
export type RenderFramePhases = {
  /**
   * Moving the cameras and weathers on, feeding the scenes what the world posted, and uploading the textures that
   * arrived.
   */
  update: number | null;
  /** Waiting for the window's next image. */
  acquire: number | null;
  /** Asking for what each view draws with: its overlays, its weather's textures and the heights its parallax reads. */
  load: number | null;
  /** Readying the frame: its uniforms, culling arguments, lights, particles and water. */
  prepare: number | null;
  /** Recording the level's passes and the window's own pass, which every viewport's picture is drawn into. */
  record: number | null;
  /** Readying the window pass's pipelines for each window's image format. */
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
  /** Milliseconds between each two presented frames over the span, in order. */
  frameTimes: Array<number | null>;
  /** Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting. */
  cpuTime: number | null;
  /** Milliseconds of the render thread's own work each frame over the span, in order. */
  cpuTimes: Array<number | null>;
  /**
   * Mean milliseconds the GPU spent on a frame from its first command to its last, whether or not its passes are
   * timed; none where the device cannot time it.
   */
  gpuTime: number | null;
  /** Milliseconds the GPU spent on each frame read back over the span, in order; each comes back a few frames late. */
  gpuTimes: Array<number | null>;
  /** Where the render thread's time goes a frame, waiting for the window's image and presenting included. */
  phases: RenderFramePhases;
  /** What each encode group cost the render thread a frame, in the order the frame records them. */
  groups: Array<RenderGroupCost>;
  /** Drawn width, in device pixels. */
  width: number;
  /** Drawn height, in device pixels. */
  height: number;
  /** The scene's width as rendered, smaller than the drawn one where it is upscaled. */
  renderWidth: number;
  /** And its height. */
  renderHeight: number;
  /** The width the frame is resolved at before it is rescaled to the drawn one. */
  outputWidth: number;
  /** And its height. */
  outputHeight: number;
  /** What smooths the frame, and upscales it where it resolves temporally. */
  antialiasing: RenderAntialiasing;
  /** Whether DLSS reconstructs rays as it upscales the frame. */
  isRayReconstructed: boolean;
  /** The graphics API drawn with. */
  backend: RenderBackend;
  /** The GPU drawn on. */
  adapter: string;
  /**
   * Whether it traces rays, by hardware ray query: what the reflections' and the water's tracing of what the screen
   * misses needs.
   */
  hasRayQueries: boolean;
  /** Whether it upscales with DLSS, and what it lacks where it does not. */
  dlss: RenderDlssAvailability;
  /** Whether it reconstructs rays with DLSS, and what it lacks where it does not. */
  rayReconstruction: RenderDlssAvailability;
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
  /** The view's grass, empty without a level or grass. */
  grass: RenderGrassReport;
  /** Milliseconds the last sector taken in took to put into the scene, on the render thread. */
  sectorTime: number | null;
  /** What the renderer holds on the GPU. */
  memory: RenderMemoryReport;
  /** What the view's spot meter read, a frame or more late; none while it does not meter. */
  meter: RenderMeterReading | null;
  /** What the frame graph made of its latest frame, none before its first. */
  graph: RenderGraphReport | null;
};

/** How glass is drawn. */
export enum ERenderGlassMode {
  /** The engine's own: its base blended over what is behind it, mixed toward the sky's cube where it is thin. */
  ENGINE = "engine",
  /**
   * Physically based glass: what stands behind it refracted, blurred where it is dirty, the sky reflected by the
   * fresnel and the sun's highlight on it.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderGlassMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderGlassMode = `${ERenderGlassMode}`;

/** How glass is drawn, and the enhanced glass's strengths. */
export type RenderGlassSettings = {
  mode: RenderGlassMode;
  /** How far the enhanced glass bends what is behind it, one as designed; none draws it flat. */
  refraction: number | null;
};

/**
 * A GPU identified by its PCI vendor and device IDs. D3D12 and Vulkan report the same IDs; two identical cards share
 * one identity.
 */
export type RenderGpu = {
  vendor: number;
  device: number;
};

/** A GPU a backend can draw on: its identity, its name and its driver. */
export type RenderGpuDescription = {
  gpu: RenderGpu;
  name: string;
  /** The driver and its version as the API reports them, for example `NVIDIA 617.14`; none if it reports neither. */
  driver: string | null;
};

/**
 * How the physical light's frame is graded before its display transform, in scene-referred light: its contrast about
 * middle grey, a power of its luminance in stops, and its saturation, a mix from its own luminance; one leaves each.
 */
export type RenderGradingSettings = {
  contrast: number | null;
  saturation: number | null;
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
 * Which of the frame graph's mechanisms every frame compiles with, each to be turned off alone, or all for serial
 * mode, to bisect a difference in a capture.
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

/** What a view's grass came to on a recent frame: the tufts its cull kept and the most its lists hold. */
export type RenderGrassReport = {
  /** Tufts the cull kept, read back a frame or more late; past `capacity` the rest went undrawn. */
  drawn: number;
  /** Tufts the lists hold at most, none while no grass is drawn. */
  capacity: number;
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
  /**
   * Degrees of slope past which a tuft is not planted, thinning over the last fifth: ninety for none, as the engine
   * plants on any ground.
   */
  slopeLimit: number | null;
  /** Whether a tuft stands along its ground's normal rather than straight up. */
  isAligned: boolean;
  /**
   * Whether the grass casts shadows at all: the contact shadows' within `shadow_radius`, and as cards (`GrassCards`)
   * the sun's and the lamps' traced under ray reconstruction or a traced occlusion, the sun's drawn into its
   * `shadow_cascades` nearest cascades otherwise, as Screen Space Shaders' `ssfx_grass_shadows` draws them. Neither
   * engine shadows its grass.
   */
  isShadowed: boolean;
  /**
   * Metres from the camera the grass casts shadows to, 150 at most, fading over the last fifth of them: the cards' as
   * far as their cascades reach, the contact shadows' past them.
   */
  shadowRadius: number | null;
  /** How many of the sun's nearest cascades the cards are drawn into: none for the contact shadows alone. */
  shadowCascades: number;
};

/**
 * What one of the frame graph's encode groups cost the render thread a frame, mean milliseconds over a report's span:
 * the groups record and are encoded at once on threads of their own, so the longest bounds the frame's encoding.
 */
export type RenderGroupCost = {
  name: string;
  /** Recording its passes. */
  record: number | null;
  /** wgpu's encoding of what was recorded, in the encoder's `finish`. */
  finish: number | null;
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
  /** Steps the search takes past the occlusion's radius out to the light's. */
  steps: number;
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

/** Graphics libraries shipped beside the application; currently only the DLSS runtime. */
export type RenderLibraries = {
  /**
   * The DLSS runtime's file version, for example `310.9.1.0`; none if the build has no DLSS or the library is missing
   * from the bundle.
   */
  dlss: string | null;
};

/**
 * How bright the physical light's sources stand against each other and the day: as the weathers and the lamps are
 * authored, or brought to the real world's.
 */
export enum ERenderLightLevel {
  /**
   * The weathers' and the lamps' own levels in the physical light's units: a night as bright as the game authors it,
   * a lamp a thousand times a real one.
   */
  AUTHORED = "authored",
  /**
   * The real world's: the weather's light dimmed with its sky towards a moonlit night of a fraction of a lux, the lamps
   * and the self-lit surfaces brought to real lamps', and the exposure held from showing the dark more than a few stops
   * under middle grey.
   */
  PHYSICAL = "physical",
}

/** Every `ERenderLightLevel` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderLightLevel = `${ERenderLightLevel}`;

/** What a view's light is computed in. */
export enum ERenderLightModel {
  /**
   * The engine's own: textures, the weather's colours and the lamps lit together as the display-encoded values they
   * are stored as, as every X-Ray renderer lights them.
   */
  ENGINE = "engine",
  /**
   * Linear light: colours authored as display values decoded before they are lit (textures and the irradiance cubes by
   * the sRGB display's 2.2 power, the weather's and the lamps' colours by their hue alone, keeping the strength the
   * game balanced them at), the scene held as linear light, and the frame shown through a display transform.
   */
  PHYSICAL = "physical",
}

/** Every `ERenderLightModel` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderLightModel = `${ERenderLightModel}`;

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
  /** Of those, lights drawing their shafts. */
  volumetric: number;
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
  /** Whether shadowed lights draw their shafts (`r2_volumetric_lights`), as the view's engine gives them out. */
  isVolumetric: boolean;
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
  /** The scene it reports on, as showing the level named it. */
  scene: number;
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

/** A level map asked of a viewport: an area in engine space seen straight down, `width` by `height` pixels. */
export type RenderMapCapture = {
  /** The area's least `x` and `z`, the map's left and bottom edges. */
  minX: number | null;
  minZ: number | null;
  /** Its greatest `x` and `z`, the map's right and top edges. */
  maxX: number | null;
  maxZ: number | null;
  /** The heights between which the level is drawn: what stands below `bottom` or above `top` is cut away. */
  bottom: number | null;
  top: number | null;
  /** The map's pixels across and down. */
  width: number;
  height: number;
  /** Times over on each side a tile is drawn before it is reduced to the map's pixels. */
  supersampling: number;
  /** Frames each tile is drawn from where it stands before the one kept, for what builds over frames to settle. */
  settleFrames: number;
  /** Whether the grass is planted over each tile, which `CDemoRecord`'s map draws none of: an experiment. */
  isGrassed: boolean;
};

/** What the renderer holds on the GPU, and what the GPU's own memory holds and allows. */
export type RenderMemoryReport = {
  /**
   * Bytes of the GPU's own memory, of it what the system lets this process hold now, and what the process holds there,
   * everything counted (DLSS's and the driver's with the renderer's); none where the backend or driver says nothing.
   */
  total: number | null;
  budget: number | null;
  used: number | null;
  /**
   * Bytes the GPU allocator holds allocated for everything the renderer made, textures, buffers and targets of every
   * viewport, and those it reserved in its blocks; none where the backend reports none.
   */
  allocated: number | null;
  reserved: number | null;
  /** The most it held allocated since the renderer started; none where the backend reports none. */
  peakAllocated: number | null;
  /**
   * What it holds allocated by the names the resources were made under, the largest first; none listed where the
   * backend reports none.
   */
  sections: Array<RenderMemorySection>;
};

/** A share of what the GPU allocator holds allocated: the resources made under one name, their bytes and how many. */
export type RenderMemorySection = {
  /** The name the resources were made under, their trailing number dropped; what the largest leave over is "Other". */
  name: string;
  bytes: number;
  count: number;
};

/**
 * What a view's spot meter read at its centre, a square of nine texels a side averaged: the light there, what it was
 * exposed by, and what the G-buffer holds of the surfaces under it.
 */
export type RenderMeterReading = {
  /**
   * The scene's luminance at the spot before its exposure, in the units its light is drawn in: cd/m² under the
   * physical light.
   */
  luminance: number | null;
  /** Stops the exposed spot stands above middle grey, 0.18; below it where negative. */
  stops: number | null;
  /** The scale the frame's light was exposed by. */
  exposure: number | null;
  /** The exposure value at ISO 100 a reflected-light meter would read at the spot, where the light is physical. */
  ev100: number | null;
  /** The surfaces' albedo luminance read as linear light (sRGB decoded); none where the spot holds only sky. */
  albedo: number | null;
  /** The surfaces' gloss, as the G-buffer keeps it. */
  gloss: number | null;
  /** The luminance of the diffuse light the sun and the lamps accumulated over the surfaces. */
  diffuse: number | null;
  /** The specular the sun and the lamps accumulated over the surfaces. */
  specular: number | null;
  /** Metres from the camera to the surfaces, along its view. */
  distance: number | null;
};

/** Every `kind` the `RenderOverlay` union is told apart by, so a switch or a comparison names one. */
export enum ERenderOverlay {
  /** Line segments in renderer space, each with a colour at either end. */
  LINES = "lines",
  /** A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting. */
  SUN = "sun",
  /** Points in renderer space, drawn as discs of one colour `size` device pixels across wherever they stand. */
  POINTS = "points",
  /** Every skinned model's bones as segments, child to parent, following its pose. */
  SKELETON = "skeleton",
}

/** A helper drawn over a viewport's frame, unlit, as the raw colours it names. */
export type RenderOverlay =
  /** Line segments in renderer space, each with a colour at either end. */
  | {
      kind: "lines";
      segments: Array<RenderOverlaySegment>;
      /** Whether what the scene draws in front hides them. */
      isDepthTested: boolean;
    }
  /** A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting. */
  | { kind: "sun"; color: [number | null, number | null, number | null]; size: number | null }
  /** Points in renderer space, drawn as discs of one colour `size` device pixels across wherever they stand. */
  | {
      kind: "points";
      positions: Array<[number | null, number | null, number | null]>;
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

/** One line segment of an overlay: its two ends in renderer space, and the raw colour at each. */
export type RenderOverlaySegment = {
  ends: [[number | null, number | null, number | null], [number | null, number | null, number | null]];
  colors: [[number | null, number | null, number | null], [number | null, number | null, number | null]];
};

/**
 * What the page shows where it is transparent around its viewports: the boxes painting there, and the wash the washed
 * ones carry.
 */
export type RenderPageBackdrop = {
  /** Outermost first, each painted over the ones before it; the first fills the window whatever its box. */
  planes: Array<RenderPagePlane>;
  wash: RenderPageWash | null;
};

/** A box of the page painting a background under a viewport, which the renderer paints in its place. */
export type RenderPagePlane = {
  /** The box its background shows in, cut to the boxes clipping it, in device pixels of the window's client area. */
  rect: RenderRect;
  /**
   * The radii its corners are rounded by in device pixels, in CSS's order: top left, top right, bottom right, bottom
   * left.
   */
  radii: [number | null, number | null, number | null, number | null];
  color: RenderColor;
  /** Whether the page's wash is laid over its colour. */
  isWashed: boolean;
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

/** Whose parallax displaces the level's surfaces. */
export enum ERenderParallaxMode {
  /** The engines' own: steep parallax on a level surface whose texture asks for it, none on terrain. */
  ENGINE = "engine",
  /**
   * Terrain marched over its details' heights, each layer by its own, self-shadowed towards the sun, its layers
   * blended by height.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderParallaxMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderParallaxMode = `${ERenderParallaxMode}`;

/** How the level's surfaces are displaced by their height. */
export type RenderParallaxSettings = {
  mode: RenderParallaxMode;
  /** `r2_steep_parallax`: the engines' steep parallax on a level surface whose texture asks for it. */
  isSteep: boolean;
  /** Steps an enhanced march takes at most, looking along a surface; three looking straight down at it. */
  steps: number;
  /** Metres within which the enhanced parallax is marched, faded over the last fifth of them. */
  range: number | null;
  /** How deep the enhanced parallax reaches, in the detail's texture coordinates. */
  height: number | null;
  /** Whether an enhanced march settles where it crossed the surface by halving its last step three times. */
  isRefined: boolean;
  /** How much of the sun the enhanced parallax's own relief keeps from its hollows: none to all. */
  shadow: number | null;
  /**
   * How far a terrain's layers blend by height, the one standing highest showing through: none, the mask alone, to
   * all.
   */
  blending: number | null;
  /** How much the enhanced parallax darkens the low ground of its relief: none to its whole share. */
  cavity: number | null;
  /**
   * How deep the enhanced parallax reaches on a level surface whose texture asks for parallax, in its own texture
   * coordinates, and the metres within which it is marched there.
   */
  surfaceHeight: number | null;
  surfaceRange: number | null;
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

/** How the falling rain is drawn. */
export enum ERenderRainDropsMode {
  /** The engine's streaks: its rain texture tinted by the weather's rain colour. */
  ENGINE = "engine",
  /**
   * Thin drops falling slower where the camera looks, each bending a blurred copy of the scene behind it and lit by a
   * glint of the light about it and the sky's brightness, and splashes bending what they land on.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderRainDropsMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderRainDropsMode = `${ERenderRainDropsMode}`;

/** Whether puddles gather on the ground when it rains, over the engine's own wetting of the surfaces either way. */
export enum ERenderRainMode {
  /** The engine's own wetting alone: splashes and running water near the camera, wet only while it rains. */
  ENGINE = "engine",
  /**
   * Puddles gathering on flat, low terrain over the engine's own wetting, filling with the rain and draining after,
   * their water reflecting as the reflections trace.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderRainMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderRainMode = `${ERenderRainMode}`;

/**
 * How rain falls and wets the frame's surfaces: the engine's streaks or refracting drops, the engine's own wetting,
 * and whether puddles gather over it, with their strengths.
 */
export type RenderRainSettings = {
  mode: RenderRainMode;
  /** How the rain falls, apart from how it wets. */
  drops: RenderRainDropsMode;
  /** Metres across the square about the camera the refracting drops fall over, thinning out past 15 metres. */
  dropSquare: number | null;
  /**
   * Metres across the ground out to which the refracting drops' rings ripple water and puddles, fading over the
   * last fifth.
   */
  ringDistance: number | null;
  /** How strongly the refracting drops' rings ripple water and puddles, of their designed strength. */
  ringStrength: number | null;
  /** How wide the refracting drops' rings spread and how long their waves are, of their designed size. */
  ringSize: number | null;
  /** How many puddles the level's sites hold: none at nought, about seven in ten at one, nearly all at two. */
  puddles: number | null;
  /** How much of a reflection a puddle takes at most. */
  reflectivity: number | null;
  /** How strongly rain ripples wet surfaces and puddles. */
  ripples: number | null;
  /**
   * How many times the game's own pace the level wets, dries and fills and drains its puddles at: one as the game
   * does, more to see a rain's course in moments.
   */
  pace: number | null;
  /** Metres out to which puddles are drawn, fading over the last sixth. */
  distance: number | null;
  /**
   * Whether the puddles fill the relief where the enhanced parallax marches the ground: water in its hollows first,
   * and the puddles' shores along them.
   */
  isReliefFlooded: boolean;
};

/** Which of DLSS ray reconstruction's models denoises and upscales: NVIDIA's pick, or one for every quality. */
export enum ERenderRayReconstructionPreset {
  /** NVIDIA's pick, which a driver update may change. */
  DEFAULT = "default",
  /** The transformer model. */
  D = "d",
  /** The latest transformer model. */
  E = "e",
  /** NVIDIA's default model, RR2. */
  F = "f",
}

/** Every `ERenderRayReconstructionPreset` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderRayReconstructionPreset = `${ERenderRayReconstructionPreset}`;

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
 * How fine the surroundings cube is, the level around the camera a reflection reads where it meets nothing on the
 * screen: texels a side of each of its six faces. Finer is sharper far reflections for more memory and a larger face
 * drawn and resolved each frame.
 */
export enum ERenderReflectionProbe {
  /** 256 texels a side: 12 MB, a soft probe. */
  COARSE = "coarse",
  /** 512 texels a side: 50 MB, a seventh of the frame's detail at the default field of view. */
  FINE = "fine",
  /** 1,024 texels a side: 200 MB, a planar reflection's spend. */
  FINEST = "finest",
}

/** Every `ERenderReflectionProbe` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderReflectionProbe = `${ERenderReflectionProbe}`;

/**
 * What the frame reflects, in three parts: the surfaces' screen-space reflections (the ground, puddles, wet and glossy
 * surfaces), the enhanced water's own reflection of the screen (rivers and lakes), and the ray trace both take what the
 * screen misses from.
 */
export type RenderReflectionSettings = {
  /** The surfaces' reflections: the engine's sky cube alone, or traced over the screen. */
  mode: RenderReflectionMode;
  /** What scales the specular's share of what a surface reflects, one as its material reflects. */
  intensity: number | null;
  /** Metres a reflection ray goes at most, the surfaces' and the water's traces alike. */
  distance: number | null;
  /**
   * Metres out to which the rain wets the ground's gloss for the rays to trace, fading over the last fifth; the
   * engine alone wets it over 25 m.
   */
  reach: number | null;
  /** The size the surfaces' rays are traced at. */
  traceSize: RenderTraceSize;
  /** Cells of the depth pyramid a surface's ray crosses at most, half of them for a rough one. */
  steps: number;
  /** How fine the surroundings cube a ray meeting nothing on the screen reads is. */
  probe: RenderReflectionProbe;
  /**
   * How far the sky's highlights are scaled at its white as light for the traced reflections, one as the engine's sky
   * picture shows them.
   */
  skyGain: number | null;
  /**
   * Whether the rays meet the grass: walked over the depth as the grass left it, rather than as it stood before the
   * grass drew, so blades and tufts stand in the reflections as solid as they draw, and stop rays behind them.
   */
  isGrassReflected: boolean;
  /**
   * The roughness past which a surface is not traced and reflects the prefiltered sky alone, the traced reflection
   * fading into it over the last third: 0.6 by default, 0.4 as Unreal's Lumen traces.
   */
  traceRoughness: number | null;
  /**
   * Whether a dry level's surfaces are traced where they show little of what they reflect, a dielectric's 4%; off,
   * they reflect the prefiltered sky alone until the rain wets them, and only metal and glass-like surfaces trace.
   */
  isDryTraced: boolean;
  /**
   * Whether each quad of traced pixels takes rays by what it shows, as Battlefield V's and AMD's SSSR's budgets do:
   * every pixel where a bright, sharp reflection or a puddle stands, two where it is dim or blurred, one where both or
   * rough, the rest filled from their neighbours; off, every traced pixel walks its own ray.
   */
  isRayBudgeted: boolean;
  /** Whether the enhanced water reflects the screen; off, it reflects the sky cube alone and keeps the rest of its look. */
  isWaterReflected: boolean;
  /** Steps the water's reflection marches over the screen. */
  waterSteps: number;
  /**
   * Whether a ray the screen does not answer is traced against the level's static geometry by hardware ray query,
   * its hit lit, in place of the sky: the surfaces' while they are drawn, the water's while it reflects; a device
   * without ray queries reflects the sky.
   */
  isTraced: boolean;
  /** Frames a pixel the screen does not answer waits between its traced rays, the last answer held between. */
  traceInterval: number;
};

/**
 * How much smaller than the viewport the scene is drawn and then upscaled: the upscalers' quality modes, by the ratio
 * of the viewport's side to the drawing's.
 */
export enum ERenderScale {
  NATIVE = "native",
  /** 1.3: about three quarters of each side, between drawing whole and Quality. */
  ULTRA_QUALITY = "ultraQuality",
  /** 1.5: two thirds of each side. */
  QUALITY = "quality",
  /** 1.7. */
  BALANCED = "balanced",
  /** 2: half of each side. */
  PERFORMANCE = "performance",
  /** 3: a third of each side. */
  ULTRA_PERFORMANCE = "ultraPerformance",
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
  /**
   * The graphics API to draw with, none for the first that starts; one that cannot start falls back to the first that
   * does, and `XRF_RENDER_BACKEND` overrides it.
   */
  backend: RenderBackend | null;
  /**
   * The GPU to draw on, none for the one the backend picks automatically (its high-performance adapter); one the
   * backend does not have falls back to automatic.
   */
  gpu: RenderGpu | null;
  /**
   * Milliseconds of GPU time each frame gives at least the work drawn once in so many frames that may wait: the
   * shadows' cascades and faces, the overhead maps, the surroundings' faces, the sky's refill. A frame held to a cap
   * or the display's refresh gives it what headroom it leaves, up to `PeriodicBudget::CEILING`.
   */
  periodicBudget: number | null;
  /** How the level's surfaces sample their textures, which every viewport shares. */
  textureFiltering: RenderTextureFiltering;
  /**
   * `texture_lod`, nought to four: how much coarser than stored the textures load, the engine's texture quality,
   * which every viewport shares.
   */
  textureLod: number;
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
  /** How far in from a cascade's edge, as a share of its width, the next cascade is mixed in. */
  blend: number | null;
  /** Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not. */
  isStaggered: boolean;
  /**
   * Times a second the cascades redraw their swaying trees at most: 60 by default, fewer to redraw them less, their
   * shadows then stepping between the sway's poses.
   */
  swayRate: number | null;
  /**
   * How many of the nearest cascades redraw their trees as they sway: all four by default, fewer to keep the farther
   * ones' trees as they were last drawn, as engines stop animating distant foliage's shadows.
   */
  swayCascades: number;
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
  /**
   * Whether calibration props stand ahead of where the camera stood when they were first shown: a grey sphere of a
   * grey card's albedo, a white one, a metal mirror and a colour chart, lit as the level's surfaces are.
   */
  isCalibrationVisible: boolean;
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
  /**
   * Steps along a ray `accum_volumetric_sun` takes; the engines' `r2_sunshafts_quality` steps OpenXRay's 20, 20 and
   * 40, Monolith's 15, 25 and 30.
   */
  steps: number;
  /** `r2_sunshafts_min`, from zero to a half: the floor the keyframes' density is lifted from; zero draws it as it is. */
  minimum: number | null;
};

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

/**
 * How a level's surfaces sample their textures: trilinear, or anisotropic at up to a number of samples along the
 * direction a texture is seen squashed, which keeps ground and walls seen at grazing angles sharp.
 */
export enum ERenderTextureFiltering {
  TRILINEAR = "trilinear",
  ANISOTROPIC2 = "anisotropic2",
  ANISOTROPIC4 = "anisotropic4",
  ANISOTROPIC8 = "anisotropic8",
  ANISOTROPIC16 = "anisotropic16",
}

/** Every `ERenderTextureFiltering` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderTextureFiltering = `${ERenderTextureFiltering}`;

/** What of a viewport's scene samples a texture. */
export enum ERenderTextureGroup {
  /** The static surfaces: the level's geometry and what stands on it. */
  SURFACES = "surfaces",
  /** The grass's tufts, and none of the surfaces. */
  GRASS = "grass",
}

/** Every `ERenderTextureGroup` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderTextureGroup = `${ERenderTextureGroup}`;

/** What became of one texture reference a viewport's scene samples. */
export type RenderTextureReport = {
  reference: string;
  state: RenderTextureState;
  /** What of the scene samples it. */
  group: RenderTextureGroup;
};

/** Every `kind` the `RenderTextureState` union is told apart by, so a switch or a comparison names one. */
export enum ERenderTextureState {
  LOADING = "loading",
  /** Held for what reads it, unread until something does: a terrain's heights while no parallax marches them. */
  DEFERRED = "deferred",
  LOADED = "loaded",
  /** Its reference resolved to no file. */
  MISSING = "missing",
  /** Its file could not be read or laid out, and why. */
  FAILED = "failed",
}

/** Where one texture a viewport's scene samples stands. */
export type RenderTextureState =
  | { kind: "loading" }
  /** Held for what reads it, unread until something does: a terrain's heights while no parallax marches them. */
  | { kind: "deferred" }
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

/** Where a view's frame is tonemapped. */
export enum ERenderTonemapMode {
  /** The engine's own: combine tonemaps the lit scene, and every forward pass after it draws tonemapped colour. */
  ENGINE = "engine",
  /**
   * The scene held as linear light at its exposure through every pass and the resolve, the engine's curve applied
   * once by the window pass at the viewport's size, as an upscaler wants its input.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderTonemapMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderTonemapMode = `${ERenderTonemapMode}`;

/** The size a reflection is traced at, of the frame's: half each way, a ray for every four pixels, or full. */
export enum ERenderTraceSize {
  HALF = "half",
  FULL = "full",
}

/** Every `ERenderTraceSize` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderTraceSize = `${ERenderTraceSize}`;

/**
 * What the scene is drawn at, a share of the viewport upscaled to it, how sharply the upscaled frame is finished,
 * which model DLSS upscales with, whether DLSS reconstructs rays as it upscales, and with which model.
 */
export type RenderUpscalingSettings = {
  scale: RenderScale;
  /** RCAS's sharpness while upscaled or resolved temporally, from none to its most. */
  sharpening: number | null;
  dlssPreset: RenderDlssPreset;
  /**
   * DLSS ray reconstruction in super resolution's place: one network denoising and upscaling the frame over its
   * surfaces' guides, where the GPU has it and the scene holds linear light.
   */
  isRayReconstructed: boolean;
  rayReconstructionPreset: RenderRayReconstructionPreset;
};

/** Each of the renderer's features as a view sets it, a full set a view: one struct a feature, owned by its module. */
export type RenderViewFeatures = {
  exposure: RenderExposureSettings;
  /** Where the frame is tonemapped: in combine and each forward pass, or once by the window pass over linear light. */
  tonemap: RenderTonemapMode;
  /** What the light is computed in: the engine's display-encoded values, or linear light. */
  lightModel: RenderLightModel;
  /** How bright the physical light's sources stand against each other and the day. */
  lightLevel: RenderLightLevel;
  /**
   * How much of the weather's own sun colour the physical light levels keep in the sun's hue: nought the atmosphere's
   * alone, one the weather's.
   */
  sunTint: number | null;
  /** What shows the physical light's frame on the display. */
  displayTransform: RenderDisplayTransform;
  /** How the physical light's frame is graded before its display transform. */
  grading: RenderGradingSettings;
  bloom: RenderBloomSettings;
  /** Which bloom is drawn, and the enhanced bloom's strengths. */
  enhancedBloom: RenderEnhancedBloomSettings;
  /** How the fog is drawn beyond the weather's keys: the engine's distance fog, or the enhanced height fog. */
  fog: RenderFogSettings;
  /** Whether the sky's colour bands are smoothed. */
  debanding: RenderDebandingSettings;
  shadows: RenderShadowSettings;
  ambientOcclusion: RenderAmbientOcclusionSettings;
  /** The light the frame's surfaces bounce onto each other, searched as the ambient occlusion's settings describe. */
  indirectLight: RenderIndirectLightSettings;
  /** What glossy surfaces reflect: the sky's cube, or what the frame shows where a ray finds it. */
  reflections: RenderReflectionSettings;
  /**
   * How rain falls and wets the surfaces: the engine's streaks or refracting drops, and the engine's splashes or wet
   * surfaces and puddles that build up and dry.
   */
  rain: RenderRainSettings;
  lights: RenderLightsSettings;
  water: RenderWaterSettings;
  /** How glass is drawn: the engine's, or physically based. */
  glass: RenderGlassSettings;
  grass: RenderGrassSettings;
  /** How finely the sun's shafts step, and Monolith's floor under their density. */
  sunShafts: RenderSunShafts;
  /** How the light the air scatters is drawn: the engine's shafts, or a froxel volume. */
  volumetric: RenderVolumetricSettings;
  /** How much of the static geometry draws at a distance. */
  lod: RenderLodSettings;
  /** How the frame's edges are smoothed. */
  antialiasing: RenderAntialiasing;
  /** How the level's surfaces are displaced by their height. */
  parallax: RenderParallaxSettings;
  /** What the finished frame is corrected by. */
  corrections: RenderImageCorrections;
  /** How the game's console scales the sun, the hemisphere and the ambient. */
  lightScales: RenderLightScales;
  /** How far the baked hemisphere darkens the ambient: zero for not at all. */
  hemiStrength: number | null;
  /** Whether what the last frame's depth hides is left undrawn. */
  isOcclusionCulled: boolean;
  /** Whether the opaque surfaces' depth is drawn alone ahead of their G-buffer draws, so each pixel is shaded once. */
  isDepthPrepassed: boolean;
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
  /** Whether a spot meter reads the light at the view's centre into its frame reports. */
  isMetered: boolean;
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

/** What a view's frame is resolved at before it is put into its rectangle, and how it is upscaled to it. */
export type RenderViewOutput = {
  /** What the scene is drawn at, a share of the output, and how its upscaled frame is sharpened. */
  upscaling: RenderUpscalingSettings;
  /**
   * Device pixels the frame is resolved tall at, as wide as the viewport's shape, or `None` for the viewport's own
   * sides times `output_scale`.
   */
  outputHeight: number | null;
  /** The share of the viewport's sides the frame is resolved at while no height is asked. */
  outputScale: number | null;
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

/** How the light the air scatters towards the camera is drawn. */
export enum ERenderVolumetricMode {
  /** The engine's own: the sun's shafts marched along each pixel's ray and the shadowed lamps' slices. */
  ENGINE = "engine",
  /**
   * A froxel volume: the sun and every local light scattered in one grid along the view, through their shadows, by a
   * phase function, kept over frames.
   */
  ENHANCED = "enhanced",
}

/** Every `ERenderVolumetricMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RenderVolumetricMode = `${ERenderVolumetricMode}`;

/** How the light the air scatters is drawn, and the enhanced volume's strengths. */
export type RenderVolumetricSettings = {
  mode: RenderVolumetricMode;
  /** How strongly the enhanced volume scatters, one as designed. */
  intensity: number | null;
  /** How far the scattered light leans towards where it was going, from none (every way alike) to nearly all. */
  anisotropy: number | null;
  /** Metres along the view the enhanced volume reaches. */
  distance: number | null;
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
  /** How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`, 1/60 on the setting's step. */
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
