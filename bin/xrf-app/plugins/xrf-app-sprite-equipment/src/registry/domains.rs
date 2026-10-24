// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_sprite_equipment_domain {
  ($consumer:ident) => {
    $consumer! {
      sprite_equipment => "sprite-equipment" {
        close_sprite => crate::commands::close_sprite::sprite_equipment_close_sprite,
        get_sprite => crate::commands::get_sprite::sprite_equipment_get_sprite,
        open_sprite => crate::commands::open_sprite::sprite_equipment_open_sprite,
        reopen_sprite => crate::commands::reopen_sprite::sprite_equipment_reopen_sprite,
        pack_sprite => crate::commands::pack_sprite::sprite_equipment_pack_sprite,
      }
    }
  };
}
