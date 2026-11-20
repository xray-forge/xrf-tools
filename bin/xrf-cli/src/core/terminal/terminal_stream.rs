/// Which of the process streams a message is written to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TerminalStream {
  Stdout,
  Stderr,
}
