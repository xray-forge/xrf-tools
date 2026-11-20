pub(crate) mod gpu;
pub(crate) mod render_frame_rate;
pub(crate) mod render_graph_settings;
pub(crate) mod render_settings;
#[cfg_attr(
  not(any(test, feature = "typescript-bindings")),
  expect(
    dead_code,
    reason = "only the TypeScript bindings' settings constant and tests read the schema"
  )
)]
pub(crate) mod schema;
