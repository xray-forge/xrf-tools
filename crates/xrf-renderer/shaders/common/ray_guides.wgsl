// What every pass writing ray reconstruction's guides agrees on: the guides pass over the G-buffer, and the enhanced
// water over it where the water draws.

// Metres a reflection stands away where no ray was traced or met anything: the sky's, the prefiltered surroundings'.
const RAY_GUIDE_FAR_HIT: f32 = 1000.0;
// How far water is reconstructed from this frame rather than its history: its ripples, waves and rain move its
// reflection in place every frame, which the history would hold behind them.
const RAY_GUIDE_BIAS_WATER: f32 = 0.8;
