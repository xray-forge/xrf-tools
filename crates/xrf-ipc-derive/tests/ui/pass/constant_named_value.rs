// A constant may be named as anything the expansion itself uses.
#[allow(non_upper_case_globals)]
#[xrf_ipc::constant]
const value: u32 = 7;

fn main() {
  let exported = xrf_ipc::IpcConstant::list()
    .find(|it| it.name == "value")
    .expect("registered");

  assert_eq!((exported.value)().expect("serialized"), xrf_ipc::serde_json::json!(value));
}
