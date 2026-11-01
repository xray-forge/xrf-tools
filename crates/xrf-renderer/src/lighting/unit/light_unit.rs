/// The luminance, in cd/m², a light of one in the engine's units stands for under the physical light. X-Ray lights a
/// surface by its albedo times the light, where a sun of E lux gives it albedo times E/π cd/m²; Anomaly's clear noon sun
/// (`w_clear1`: 1, 0.8, 0.65 at `r2_sun_lumscale` 2, 3.3 decoded) then comes to about 100 000 lx, and its clear sky to a
/// few thousand cd/m², as a real one is.
pub const PHYSICAL_LIGHT_UNIT: f32 = 10_000.0;
