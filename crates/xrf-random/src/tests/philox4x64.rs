use crate::philox::philox4x64::Philox4x64;

/// Random123's known answers for Philox4x64-10 (`tests/kat_vectors`): counter, key, output.
#[test]
fn answers_random123s_known_vectors() {
  assert_eq!(
    Philox4x64::generate([0; 4], [0; 2]),
    [
      0x1655_4d9e_ca36_314c,
      0xdb20_fe9d_672d_0fdc,
      0xd7e7_72ce_e186_176b,
      0x7e68_b68a_ec7b_a23b
    ]
  );
  assert_eq!(
    Philox4x64::generate([u64::MAX; 4], [u64::MAX; 2]),
    [
      0x87b0_92c3_013f_e90b,
      0x438c_3c67_be8d_0224,
      0x9cc7_d7c6_9cd7_77b6,
      0xa09c_aebf_594f_0ba0
    ]
  );
  assert_eq!(
    Philox4x64::generate(
      [
        0x243f_6a88_85a3_08d3,
        0x1319_8a2e_0370_7344,
        0xa409_3822_299f_31d0,
        0x082e_fa98_ec4e_6c89
      ],
      [0x4528_21e6_38d0_1377, 0xbe54_66cf_34e9_0c6c]
    ),
    [
      0xa528_f454_03e6_1d95,
      0x38c7_2dbd_566e_9788,
      0xa5a1_610e_72fd_18b5,
      0x57bd_43b5_e52b_7fe6
    ]
  );
}
