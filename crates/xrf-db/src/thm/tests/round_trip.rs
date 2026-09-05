use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;

use crate::thm::chunks::thm_extra_chunk::ThmExtraChunk;
use crate::thm::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use crate::thm::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
use crate::thm::tests::fixtures;
use crate::thm::thm_file::ThmFile;

/// Read a descriptor and write it straight back.
fn rewrite(bytes: &[u8]) -> XrfResult<(ThmFile, Vec<u8>)> {
  let file: ThmFile = ThmFile::read_from_bytes::<XRayByteOrder>(bytes.to_vec())?;
  let written: Vec<u8> = file.write_to_bytes::<XRayByteOrder>()?;

  Ok((file, written))
}

#[test]
fn rewrites_a_descriptor_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = fixtures::descriptor();
  let (file, written) = rewrite(&original)?;

  assert_eq!(
    file.used_bump_name(),
    Some(fixtures::BUMP_NAME),
    "Expect the sample to declare its bump, so the rewrite is judged on a file with something in every chunk"
  );
  assert_eq!(
    written, original,
    "Expect a descriptor read and written back to be the file it was"
  );

  Ok(())
}

#[test]
fn writes_the_chunks_in_whole_file_order() -> XrfResult {
  // The order is `ETextureThumbnail::Save`'s, which is not the numeric order of the ids: the thumbnail's own type at
  // 0x0813 precedes the texture params at 0x0812, and a writer sorting by id would put them the other way round.
  let (_, written) = rewrite(&fixtures::descriptor())?;

  assert_eq!(fixtures::read_chunk_ids(&written), fixtures::WHOLE_FILE_ORDER.to_vec());

  Ok(())
}

#[test]
fn rewrites_a_descriptor_carrying_a_thumbnail_byte_for_byte() -> XrfResult {
  // The payload is the engine's own compressed stream, which nothing here can rebuild, so the only correct rewrite is
  // the one that copies it. Its id carries `CFS_CompressMark`, which has to survive too.
  let payload: Vec<u8> = (0..=u8::MAX).collect();
  let original: Vec<u8> = fixtures::descriptor_with_thumbnail(&payload);
  let (file, written) = rewrite(&original)?;
  let thumbnail: ThmThumbnailChunk = file.thumbnail.expect("Expect the thumbnail chunk to be read");

  assert!(thumbnail.is_compressed, "Expect the compression mark to be recognised");
  assert_eq!(thumbnail.data, payload, "Expect the payload to be carried through");
  assert_eq!(written, original);

  Ok(())
}

#[test]
fn keeps_a_chunk_it_cannot_fold() -> XrfResult {
  // Nothing in the corpus carries one, but an editor that dropped what it did not understand would quietly destroy
  // whatever another tool had put there.
  const UNKNOWN_ID: u32 = 0x0820;

  let original: Vec<u8> = [fixtures::descriptor(), fixtures::raw_chunk(UNKNOWN_ID, b"kept")].concat();
  let (file, written) = rewrite(&original)?;

  assert_eq!(
    file.extra,
    vec![ThmExtraChunk {
      id: UNKNOWN_ID,
      data: b"kept".to_vec()
    }],
    "Expect an id the format has no name for to be kept as it was read"
  );
  assert_eq!(written, original);

  Ok(())
}

#[test]
fn keeps_a_second_copy_of_a_known_chunk() -> XrfResult {
  // The engine's `find_chunk` stops at the first match, so a second copy is data no loader reads. It is still not
  // this crate's to throw away, and the field has to hold the copy the engine would have taken.
  let original: Vec<u8> = [
    fixtures::descriptor(),
    fixtures::raw_chunk(ThmFile::TEXTURE_TYPE_CHUNK_ID, &4_u32.to_le_bytes()),
  ]
  .concat();
  let (file, written) = rewrite(&original)?;

  assert!(
    file.texture_type().is_described_by_engine(),
    "Expect the first copy to be the one that fills the field"
  );
  assert_eq!(
    file.extra,
    vec![ThmExtraChunk {
      id: ThmFile::TEXTURE_TYPE_CHUNK_ID,
      data: 4_u32.to_le_bytes().to_vec()
    }],
    "Expect the second copy to be kept rather than to overwrite the first"
  );
  assert_eq!(written, original);

  Ok(())
}

#[test]
fn writes_nothing_for_an_absent_chunk() -> XrfResult {
  // A descriptor missing a chunk is a file the engine loads, so a rewrite that filled the gap with a default would
  // change what the file says rather than preserve it.
  let original: Vec<u8> = fixtures::raw_chunk(ThmTextureParamChunk::CHUNK_ID, &[0; 32]);
  let (file, written) = rewrite(&original)?;

  assert_eq!(file.version, None);
  assert_eq!(file.bump, None);
  assert_eq!(fixtures::read_chunk_ids(&written), vec![ThmTextureParamChunk::CHUNK_ID]);
  assert_eq!(written, original);

  Ok(())
}
