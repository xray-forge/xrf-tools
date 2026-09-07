use crate::shader_library::shader_blender::ShaderBlender;
use crate::shader_library::shader_blender_class::ShaderBlenderClass;
use crate::shader_library::shader_blender_property::ShaderBlenderProperty;
use crate::shader_library::shader_blender_property_value::ShaderBlenderPropertyValue;
use crate::shader_library::shader_blender_token::ShaderBlenderToken;

/// One blender, built as the SDK writes it and then changed a knob at a time.
///
/// Each constructor lays out the whole property grid of one class, in the order and with the spellings that class's
/// own `Save` uses, so a test reads as the difference from a shader the game ships rather than from an invented one.
/// The alpha knobs are named per class deliberately - `B_MODEL` writes `Use alpha-channel` where `B_DEFAULT_AREF`
/// writes `Alpha-blend` - because a reader that folds those spellings together cannot be caught doing it.
#[derive(Clone, Debug)]
pub struct ShaderBlenderFixture {
  pub blender: ShaderBlender,
  /// The boolean knob this class spells its alpha switch with, or `None` for a class that has none.
  alpha_property: Option<&'static str>,
}

impl ShaderBlenderFixture {
  /// `B_MODEL`, the class almost every mesh surface names: an alpha switch and a reference of its own.
  ///
  /// The grid is `models\model`'s as the corpus stores it, down to the `$base0` texture and the four tessellation
  /// tokens.
  pub fn model(name: &str) -> Self {
    Self {
      blender: Self::describe(
        ShaderBlenderClass::MODEL,
        name,
        2,
        vec![
          Self::boolean("Use alpha-channel", false),
          Self::integer("Alpha ref", 32, 0, 255),
          Self::tessellation(),
        ],
      ),
      alpha_property: Some("Use alpha-channel"),
    }
  }

  /// `B_MODEL_EbB`, the environment mapped model class: an alpha switch and no reference at all.
  pub fn model_environment(name: &str) -> Self {
    Self {
      blender: Self::describe(
        ShaderBlenderClass::MODEL_EB_B,
        name,
        1,
        vec![
          ShaderBlenderProperty {
            name: String::from("Environment map"),
            value: ShaderBlenderPropertyValue::Marker,
          },
          ShaderBlenderProperty {
            name: String::from("Name"),
            value: ShaderBlenderPropertyValue::Texture(String::from("$null")),
          },
          ShaderBlenderProperty {
            name: String::from("Transform"),
            value: ShaderBlenderPropertyValue::Matrix(String::from("$null")),
          },
          Self::boolean("Alpha-Blend", false),
        ],
      ),
      alpha_property: Some("Alpha-Blend"),
    }
  }

  /// `B_DEFAULT_AREF`, the level's alpha tested class, whose reference defaults to 200 rather than 32.
  pub fn level_aref(name: &str) -> Self {
    Self {
      blender: Self::describe(
        ShaderBlenderClass::DEFAULT_AREF,
        name,
        1,
        vec![
          Self::integer("Alpha ref", 200, 0, 255),
          Self::boolean("Alpha-blend", false),
        ],
      ),
      alpha_property: Some("Alpha-blend"),
    }
  }

  /// `B_TREE`, which cuts out from the switch alone.
  pub fn tree(name: &str) -> Self {
    Self {
      blender: Self::describe(
        ShaderBlenderClass::TREE,
        name,
        1,
        vec![Self::boolean("Alpha-blend", false), Self::boolean("Object LOD", false)],
      ),
      alpha_property: Some("Alpha-blend"),
    }
  }

  /// `B_DETAIL`, which cuts out whatever its switch says.
  pub fn detail(name: &str) -> Self {
    Self {
      blender: Self::describe(
        ShaderBlenderClass::DETAIL,
        name,
        0,
        vec![Self::boolean("Alpha-blend", false)],
      ),
      alpha_property: Some("Alpha-blend"),
    }
  }

  /// A blender of any class carrying only the properties every class writes.
  ///
  /// For the classes a surface can name and no mesh renderer draws with, and for the ones a mod's renderer added.
  pub fn of(class: ShaderBlenderClass, name: &str) -> Self {
    Self {
      blender: Self::describe(class, name, 0, Vec::new()),
      alpha_property: None,
    }
  }

  /// Sets the class's own alpha switch, whatever that class spells it.
  ///
  /// # Panics
  ///
  /// When the class writes no switch, which would otherwise make a test assert against a knob the engine never reads.
  pub fn with_alpha_channel(mut self, is_used: bool) -> Self {
    let name: &'static str = self
      .alpha_property
      .expect("Expect a class that declares an alpha switch");

    self.set(name, ShaderBlenderPropertyValue::Bool(is_used));
    self
  }

  /// Sets the class's alpha reference.
  ///
  /// # Panics
  ///
  /// When the class writes none, since one appended here would be a knob the engine cannot read.
  pub fn with_alpha_reference(mut self, reference: i32) -> Self {
    assert!(
      self.blender.find_property("Alpha ref").is_some(),
      "Expect a class that declares an alpha reference"
    );

    self.set(
      "Alpha ref",
      ShaderBlenderPropertyValue::Integer {
        value: reference,
        minimum: 0,
        maximum: 255,
      },
    );
    self
  }

  pub fn with_strict_sorting(mut self, is_strict: bool) -> Self {
    self.set(
      ShaderBlender::STRICT_SORTING_PROPERTY,
      ShaderBlenderPropertyValue::Bool(is_strict),
    );
    self
  }

  /// Drops one property, for the file a tool wrote without a knob its class defines.
  pub fn without_property(mut self, name: &str) -> Self {
    self.blender.properties.retain(|property| property.name != name);
    self
  }

  fn set(&mut self, name: &str, value: ShaderBlenderPropertyValue) {
    match self.blender.properties.iter_mut().find(|it| it.name == name) {
      Some(property) => property.value = value,
      None => self.blender.properties.push(ShaderBlenderProperty {
        name: String::from(name),
        value,
      }),
    }
  }

  /// The base grid `IBlender::Save` writes for every class, followed by the class's own.
  fn describe(class: ShaderBlenderClass, name: &str, version: u16, own: Vec<ShaderBlenderProperty>) -> ShaderBlender {
    let mut properties: Vec<ShaderBlenderProperty> = vec![
      ShaderBlenderProperty {
        name: String::from("General"),
        value: ShaderBlenderPropertyValue::Marker,
      },
      Self::integer(ShaderBlender::PRIORITY_PROPERTY, 1, 0, 3),
      Self::boolean(ShaderBlender::STRICT_SORTING_PROPERTY, false),
      ShaderBlenderProperty {
        name: String::from("Base Texture"),
        value: ShaderBlenderPropertyValue::Marker,
      },
      ShaderBlenderProperty {
        name: String::from("Name"),
        value: ShaderBlenderPropertyValue::Texture(String::from("$base0")),
      },
      ShaderBlenderProperty {
        name: String::from("Transform"),
        value: ShaderBlenderPropertyValue::Matrix(String::from("$null")),
      },
    ];

    properties.extend(own);

    ShaderBlender {
      class,
      name: String::from(name),
      computer: String::from("XRF"),
      time: 0,
      version,
      properties,
    }
  }

  fn boolean(name: &str, value: bool) -> ShaderBlenderProperty {
    ShaderBlenderProperty {
      name: String::from(name),
      value: ShaderBlenderPropertyValue::Bool(value),
    }
  }

  fn integer(name: &str, value: i32, minimum: i32, maximum: i32) -> ShaderBlenderProperty {
    ShaderBlenderProperty {
      name: String::from(name),
      value: ShaderBlenderPropertyValue::Integer {
        value,
        minimum,
        maximum,
      },
    }
  }

  fn tessellation() -> ShaderBlenderProperty {
    ShaderBlenderProperty {
      name: String::from("Tessellation"),
      value: ShaderBlenderPropertyValue::Token {
        selected: 0,
        items: ["NO_TESS", "TESS_PN", "TESS_HM", "TESS_PN+HM"]
          .iter()
          .enumerate()
          .map(|(index, label)| ShaderBlenderToken {
            id: index as u32,
            label: String::from(*label),
          })
          .collect(),
      },
    }
  }
}

impl From<ShaderBlenderFixture> for ShaderBlender {
  fn from(fixture: ShaderBlenderFixture) -> Self {
    fixture.blender
  }
}
