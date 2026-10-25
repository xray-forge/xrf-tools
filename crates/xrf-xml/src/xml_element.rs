/// One XML element with ordered attributes and child elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlElement {
  name: String,
  attributes: Vec<XmlAttribute>,
  children: Vec<XmlElement>,
  text: String,
}

impl XmlElement {
  pub(crate) fn from_node(node: roxmltree::Node<'_, '_>) -> Self {
    Self {
      name: node.tag_name().name().to_string(),
      attributes: node
        .attributes()
        .map(|attribute| XmlAttribute {
          name: attribute.name().to_string(),
          value: attribute.value().to_string(),
        })
        .collect(),
      children: node
        .children()
        .filter(|child| child.is_element())
        .map(Self::from_node)
        .collect(),
      text: node
        .children()
        .filter(|child| child.is_text())
        .filter_map(|child| child.text())
        .collect(),
    }
  }

  /// Returns the element's unqualified tag name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns the value of the first attribute with the exact name, if present.
  pub fn attribute(&self, name: &str) -> Option<&str> {
    self
      .attributes
      .iter()
      .find(|attribute| attribute.name == name)
      .map(|attribute| attribute.value.as_str())
  }

  /// Iterates over `(name, value)` pairs in source order.
  pub fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
    self
      .attributes
      .iter()
      .map(|attribute| (attribute.name.as_str(), attribute.value.as_str()))
  }

  /// Iterates over direct child elements with the exact tag name.
  pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Self> + 'a {
    self.children.iter().filter(move |child| child.name == name)
  }

  /// Iterates over descendants in document order, excluding this element.
  pub fn descendants(&self) -> impl Iterator<Item = &Self> {
    XmlDescendants {
      stack: self.children.iter().rev().collect(),
    }
  }

  /// Iterates over descendants with the exact tag name, excluding this element.
  pub fn descendants_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Self> + 'a {
    self.descendants().filter(move |element| element.name == name)
  }

  /// Returns concatenated text from this element's direct text children.
  ///
  /// Text nested inside child elements is not included.
  pub fn text(&self) -> &str {
    &self.text
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct XmlAttribute {
  name: String,
  value: String,
}

struct XmlDescendants<'a> {
  stack: Vec<&'a XmlElement>,
}

impl<'a> Iterator for XmlDescendants<'a> {
  type Item = &'a XmlElement;

  fn next(&mut self) -> Option<Self::Item> {
    let element: &XmlElement = self.stack.pop()?;
    self.stack.extend(element.children.iter().rev());

    Some(element)
  }
}
