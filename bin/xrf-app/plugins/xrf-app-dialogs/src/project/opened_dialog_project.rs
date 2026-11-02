use std::sync::Arc;

use xrf_dialog::{DialogProject, DialogProjectDescriptor};

/// A dialogs project as an open read it, with its description, made once at the open: what the page lists, which a
/// restore answers again without describing every dialog afresh.
pub struct OpenedDialogProject {
  pub project: DialogProject,
  pub descriptor: Arc<DialogProjectDescriptor>,
}

impl OpenedDialogProject {
  /// The project with its description.
  pub fn new(project: DialogProject) -> Self {
    let descriptor: Arc<DialogProjectDescriptor> = Arc::new(project.describe());

    Self { project, descriptor }
  }

  /// The description the page lists the project by.
  pub fn describe(&self) -> Arc<DialogProjectDescriptor> {
    Arc::clone(&self.descriptor)
  }
}
