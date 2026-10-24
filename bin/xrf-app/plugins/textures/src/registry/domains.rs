// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_textures_domain {
  ($consumer:ident) => {
    $consumer! {
      textures => "textures" {
        build_from_source => crate::commands::build_from_source::textures_build_from_source,
        close => crate::commands::close::textures_close,
        compare_encodings => crate::commands::compare_encodings::textures_compare_encodings,
        describe => crate::commands::describe::textures_describe,
        describe_catalog => crate::commands::describe_catalog::textures_describe_catalog,
        get_session => crate::commands::get_session::textures_get_session,
        get_vocabulary => crate::commands::get_vocabulary::textures_get_vocabulary,
        make_bump => crate::commands::make_bump::textures_make_bump,
        open => crate::commands::open::textures_open,
        save => crate::commands::save::textures_save,
        show_texture => crate::commands::show_texture::textures_show_texture,
      }
      // Decoded images the page shows: a texture or a save candidate as png, and texels; stored bytes are `assets/read_asset`.
      @bulk {
        read_candidate(sessionId: "SessionId", format: "TextureEncodingFormat") => crate::routes::read_candidate::textures_read_candidate,
        read_texels(roots: "XrayRoots", logicalPath: "string") => crate::routes::read_texels::textures_read_texels,
        read_texture(roots: "XrayRoots", logicalPath: "string") => crate::routes::read_texture::textures_read_texture,
      }
    }
  };
}
