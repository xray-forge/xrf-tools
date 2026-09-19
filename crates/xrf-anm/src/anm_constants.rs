/// Channels a motion animates, in the order the file stores them.
pub const ANM_CHANNELS: [&str; 6] = [
  "position x",
  "position y",
  "position z",
  "rotation pitch",
  "rotation heading",
  "rotation bank",
];

/// Frames a second an animation is authored at when it declares nothing usable.
///
/// Every `.anm` of the workspace trees declares 30, which is also what `CCustomMotion` starts at.
pub const ANM_DEFAULT_FPS: f32 = 30.0;
