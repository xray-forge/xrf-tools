/// Channels a motion animates, in the order the file stores them: `ctPositionX` to `ctRotationB`
/// (`xrCore/Animation/Motion.hpp`), the heading before the pitch.
pub const ANM_CHANNELS: [&str; 6] = [
  "position x",
  "position y",
  "position z",
  "rotation heading",
  "rotation pitch",
  "rotation bank",
];

/// Frames a second an animation is authored at when it declares nothing usable.
///
/// Every `.anm` of the workspace trees declares 30, which is also what `CCustomMotion` starts at.
pub const ANM_DEFAULT_FPS: f32 = 30.0;
