/// One XML element with ordered attributes and child elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlElement {
  name: String,
  attributes: Vec<(String, String)>,
  children: Vec<XmlElement>,
  text: String,
}

impl XmlElement {
  pub(crate) fn from_node(node: roxmltree::Node<'_, '_>) -> Self {
    let mut children: Vec<Self> = Vec::new();

    // Recursing from a plain loop keeps each level's stack to this one frame, rather than the iterator frames `collect`
    // stacks between levels in an unoptimized build.
    for child in node.children().filter(roxmltree::Node::is_element) {
      children.push(Self::from_node(child));
    }

    Self {
      name: node.tag_name().name().to_owned(),
      attributes: node
        .attributes()
        .map(|attribute| (attribute.name().to_owned(), attribute.value().to_owned()))
        .collect(),
      children,
      text: node
        .children()
        .filter(roxmltree::Node::is_text)
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
      .find(|(attribute, _)| attribute == name)
      .map(|(_, value)| value.as_str())
  }

  /// Iterates over `(name, value)` pairs in source order.
  pub fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
    self
      .attributes
      .iter()
      .map(|(name, value)| (name.as_str(), value.as_str()))
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
