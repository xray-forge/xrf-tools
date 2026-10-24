use crate::window::WindowCloseApprovals;

#[test]
fn an_approval_lets_one_close_of_its_window_through() {
  let approvals: WindowCloseApprovals = WindowCloseApprovals::default();

  assert!(!approvals.take("main"));

  approvals.approve("main");

  assert!(!approvals.take("other"));
  assert!(approvals.take("main"));
  assert!(!approvals.take("main"));
}
