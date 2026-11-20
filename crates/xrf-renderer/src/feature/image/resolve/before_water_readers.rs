/// What besides the temporal resolve reads the frame as it stood before the water: ray reconstruction, telling the
/// transparent layers apart by it where it reads its guides, and the enhanced water, which refracts and reflects it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BeforeWaterReaders {
  pub is_guided: bool,
  pub is_water_reading: bool,
}
