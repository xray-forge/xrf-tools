use crate::scene::texture::texture_role::TextureRole;

#[test]
fn texture_roles_are_listed_in_declaration_order() {
  for (index, role) in TextureRole::ALL.iter().enumerate() {
    assert_eq!(*role as usize, index);
  }

  assert_eq!(TextureRole::ALL.len(), TextureRole::Projector as usize + 1);
}
