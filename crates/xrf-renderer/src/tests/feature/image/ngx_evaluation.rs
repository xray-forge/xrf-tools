use crate::feature::image::resolve::dlss::ngx_evaluation::NgxEvaluation;

#[test]
fn answers_an_evaluation_ngx_took() {
  let mut evaluation: NgxEvaluation = NgxEvaluation::default();

  assert_eq!(evaluation.keep(Ok::<u32, &str>(7)), Some(7));
  assert_eq!(evaluation.take_refusal(), None);
}

// Only the first refusal matters: the prepare hands the frame to another resolve on it.
#[test]
fn keeps_the_first_refusal_for_the_prepare_once() {
  let mut evaluation: NgxEvaluation = NgxEvaluation::default();

  assert_eq!(evaluation.keep(Err::<u32, &str>("refused")), None);
  assert_eq!(evaluation.keep(Err::<u32, &str>("refused again")), None);
  assert_eq!(evaluation.keep(Ok::<u32, &str>(7)), Some(7));
  assert_eq!(evaluation.take_refusal().as_deref(), Some("refused"));
  assert_eq!(evaluation.take_refusal(), None);
}
