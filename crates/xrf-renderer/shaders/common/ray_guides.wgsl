// What every pass writing ray reconstruction's guides agrees on: the guides pass over the G-buffer, and the enhanced
// water over it where the water draws.

// Metres a reflection stands away where no ray was traced or met anything: the sky's, the prefiltered surroundings'.
const RAY_GUIDE_FAR_HIT: f32 = 1000.0;
