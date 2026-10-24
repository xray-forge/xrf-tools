// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_textures_domain {
  ($consumer:ident) => {
    $consumer! {
      textures => "textures" {
        build_from_source => crate::edit::commands::build_from_source::textures_build_from_source,
        close_browse => crate::browse::commands::close_browse::textures_close_browse,
        close_comparison => crate::encoding::commands::close_comparison::textures_close_comparison,
        compare_encodings => crate::encoding::commands::compare_encodings::textures_compare_encodings,
        describe => crate::describe::commands::describe::textures_describe,
        describe_catalog => crate::browse::commands::describe_catalog::textures_describe_catalog,
        get_vocabulary => crate::describe::commands::get_vocabulary::textures_get_vocabulary,
        make_bump => crate::edit::commands::make_bump::textures_make_bump,
        open_browse => crate::browse::commands::open_browse::textures_open_browse,
        restore_browse => crate::browse::commands::restore_browse::textures_restore_browse,
        save => crate::edit::commands::save::textures_save,
        show_texture => crate::surface::commands::show_texture::textures_show_texture,
      }
      // Pictures the page shows that only textures makes: a save candidate as png, and texels; a texture as png is
      // `assets/read_texture`, and stored bytes are `assets/read_asset`.
      @bulk {
        read_candidate(sessionId: "SessionId", format: "TextureEncodingFormat") => crate::encoding::routes::read_candidate::textures_read_candidate,
        read_texels(roots: "XrayRoots", source: "AssetSource") => crate::describe::routes::read_texels::textures_read_texels,
      }
      @jobs {
        /// Rebuilding a texture from a source image, as its descriptor says to.
        TEXTURES_JOB_BUILD => "build",
        /// Weighing every candidate format against one texture.
        TEXTURES_JOB_COMPARE_ENCODINGS => "compare-encodings",
        /// Generating a bump pair from a height map.
        TEXTURES_JOB_MAKE_BUMP => "make-bump",
        /// Saving a node's descriptor, and its texture with it.
        TEXTURES_JOB_SAVE => "save",
      }
    }
  };
}
