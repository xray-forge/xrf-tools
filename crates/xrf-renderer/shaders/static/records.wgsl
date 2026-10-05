#import "generated/structs"

// What the static scene's records mean beyond their layout, which `generated/structs` declares from Rust.

// A surface's texture slots, by their place in its `textures`, as `StaticSurface` orders them; water's take the detail's,
// the bump's and its companion's places.
const SLOT_BASE: u32 = 0u;
const SLOT_DETAIL: u32 = 1u;
const SLOT_BUMP: u32 = 2u;
const SLOT_BUMP_COMPANION: u32 = 3u;
const SLOT_DETAIL_BUMP: u32 = 4u;
const SLOT_DETAIL_BUMP_COMPANION: u32 = 5u;
const SLOT_HEMI: u32 = 6u;
const SLOT_ENVIRONMENT: u32 = 7u;
const SLOT_WATER_NORMAL: u32 = 1u;
const SLOT_FOAM: u32 = 2u;
const SLOT_DISTORTION: u32 = 3u;

const IMPOSTOR_FACETS: u32 = 8u;

// What an impostor's level of detail draws of its clump: its trees, itself, or both while one fades into the other.
const LOD_TREES: u32 = 1u;
const LOD_IMPOSTOR: u32 = 2u;

// A row no impostor decides.
const NO_LOD: u32 = 0xffffffffu;

// A row's band word, as `StaticRow::pack_band` lays it: its progressive band and the bands its mesh has in four bits
// each, and the slide windows they share in sixteen.
fn row_band(word: u32) -> u32 {
  return word & 15u;
}

fn row_bands(word: u32) -> u32 {
  return (word >> 4u) & 15u;
}

fn row_windows(word: u32) -> u32 {
  return (word >> 8u) & 0xffffu;
}

// The shading classes a layout's batches run through, as `StaticClass` orders them: the two the G-buffer draws and
// shadows cast first, then water, composited surfaces and wall marks.
const CLASS_COUNT: u32 = 5u;
const WATER_CLASS: u32 = 2u;

const SLOT_SINGLE: u32 = 1u;
const SLOT_LISTED: u32 = 2u;

const SURFACE_HAS_BASE: u32 = 1u;
const SURFACE_HAS_DETAIL: u32 = 2u;
const SURFACE_HAS_BUMP: u32 = 4u;
const SURFACE_HAS_DETAIL_BUMP: u32 = 8u;
const SURFACE_HAS_HEMI: u32 = 16u;
const SURFACE_IS_CUT_OUT: u32 = 32u;
const SURFACE_IS_SOFT_WATER: u32 = 64u;
const SURFACE_IS_ANOMALY_WATER: u32 = 128u;
const SURFACE_IS_REFLECTING: u32 = 256u;
const SURFACE_IS_SPECULAR: u32 = 512u;
const SURFACE_IS_TRANSPARENT: u32 = 1024u;
const SURFACE_IS_FOAMED: u32 = 2048u;
const SURFACE_HAS_WATER_NORMAL: u32 = 4096u;
const SURFACE_HAS_FOAM: u32 = 8192u;
const SURFACE_HAS_DISTORTION: u32 = 16384u;
const SURFACE_IS_ADDED: u32 = 32768u;
const SURFACE_IS_WEIGHTED: u32 = 65536u;
const SURFACE_IS_MULTIPLIED: u32 = 131072u;
const SURFACE_IS_DOUBLED: u32 = 262144u;
const SURFACE_IS_MODEL: u32 = 524288u;
const SURFACE_IS_ENVIRONMENT_MAPPED: u32 = 1048576u;
// An object the level stores as a tree, which the wind leaves standing.
const SURFACE_IS_STILL: u32 = 2097152u;
// A terrain's, which lays four details and their bumps over its base by its mask, lit by its base's alpha.
const SURFACE_IS_TERRAIN: u32 = 4194304u;
// A self-lit surface's, whose light the sun pass fills.
const SURFACE_IS_EMISSIVE: u32 = 8388608u;
// A surface casting no shadow, whose script declares no shadow element.
const SURFACE_IS_SHADOWLESS: u32 = 16777216u;

// Vertices one cluster's draw spans: 128 triangles, those past its own collapsed.
const CLUSTER_VERTICES: u32 = 384u;
