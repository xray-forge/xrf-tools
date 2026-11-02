// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_sprite_equipment_domain {
  ($consumer:ident) => {
    $consumer! {
      sprite_equipment => "sprite-equipment" {
        close_sprite => crate::commands::close_sprite::sprite_equipment_close_sprite,
        open_sprite => crate::commands::open_sprite::sprite_equipment_open_sprite,
        pack_sprite => crate::commands::pack_sprite::sprite_equipment_pack_sprite,
        reload_sprite => crate::commands::reload_sprite::sprite_equipment_reload_sprite,
        restore_sprite => crate::commands::restore_sprite::sprite_equipment_restore_sprite,
      }
      @bulk {
        read_preview(sessionId: "SessionId") => crate::routes::read_preview::sprite_equipment_read_preview,
      }
      @jobs {
        /// Packing loose inventory icons into an equipment sprite sheet.
        SPRITE_EQUIPMENT_JOB_PACK => "pack",
      }
    }
  };
}
