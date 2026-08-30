use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfProgressive {}

impl ChunkReadWrite for OgfProgressive {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}

/*
sub read_progressive {
  my $self = shift;
  my ($cf) = @_;
  $self->read_visual($cf);
  if ($self->{ogf_version} == 4 && $cf->find_chunk($chunk_names{$self->{ogf_version}}{'OGF_SWIDATA'})) {
    $self->read_swidata($cf);
    $cf->close_found_chunk();
  } else {
    if ($cf->find_chunk($chunk_names{$self->{ogf_version}}{'OGF_LODDATA'})) {
      $self->read_loddata($cf);
      $cf->close_found_chunk();
    } else {
      fail('Invalid visual, no loddata');
    }
  }
}
 */
