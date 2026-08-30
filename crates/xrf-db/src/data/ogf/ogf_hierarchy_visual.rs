use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfHierarchyVisual {}

impl ChunkReadWrite for OgfHierarchyVisual {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}

/*

sub read_hierrarhy_visual {
  my $self = shift;
  my ($cf) = @_;
  $self->read_render_visual($cf);
  if ($cf->find_chunk($chunk_names{$self->{ogf_version}}{'OGF_CHILDREN_L'})) {
    $self->read_children_l($cf);
  } elsif ($self->{ogf_version} != 2 && $cf->find_chunk($chunk_names{$self->{ogf_version}}{'OGF_CHILDREN'})) {
    $self->read_children($cf);
  } elsif ($cf->find_chunk($chunk_names{$self->{ogf_version}}{'OGF_CHILD_REFS'})) {
    $self->read_child_refs($cf);
  } else {
    fail('Invalid visual, no children');
  }
  $cf->close_found_chunk();
};
 */
