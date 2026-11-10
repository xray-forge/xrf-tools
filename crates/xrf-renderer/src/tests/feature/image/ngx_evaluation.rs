use crate::feature::image::resolve::dlss::ngx_evaluation::NgxEvaluation;

/// An evaluation NGX took answers its commands and leaves nothing refused.
#[test]
fn answers_an_evaluation_ngx_took() {
  let mut evaluation: NgxEvaluation = NgxEvaluation::default();

  assert_eq!(evaluation.keep(Ok::<u32, &str>(7)), Some(7));
  assert_eq!(evaluation.take_refusal(), None);
}

/// A refused evaluation is kept for the prepare, which reads it once: the first of several frames refused, since the
/// prepare hands the frame to another resolve at the first.
#[test]
fn keeps_the_first_refusal_for_the_prepare_once() {
  let mut evaluation: NgxEvaluation = NgxEvaluation::default();

  assert_eq!(evaluation.keep(Err::<u32, &str>("refused")), None);
  assert_eq!(evaluation.keep(Err::<u32, &str>("refused again")), None);
  assert_eq!(evaluation.keep(Ok::<u32, &str>(7)), Some(7));
  assert_eq!(evaluation.take_refusal().as_deref(), Some("refused"));
  assert_eq!(evaluation.take_refusal(), None);
}
