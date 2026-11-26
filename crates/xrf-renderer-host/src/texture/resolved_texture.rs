/// A texture reference resolved by its render source rather than by the resolver, as a level resolves its own at open.
pub struct ResolvedTexture {
  /// The reference as the source's surfaces spell it.
  pub reference: String,
  /// The file it resolved to, or `None` for a reference resolving to nothing.
  pub logical_path: Option<String>,
}
