use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::{Body, Frame, SizeHint};
use tokio::sync::mpsc::Receiver;

use crate::core::transport::TransportPart;

/// A response's body: a route's whole answer, or a batch's parts as its calls finish.
pub(crate) enum TransportBody {
  Whole(Full<Bytes>),
  Parts {
    parts: Receiver<TransportPart>,
    /// The bytes of the part whose header went last, sent next.
    pending: Option<Bytes>,
  },
}

impl TransportBody {
  pub(crate) fn whole(bytes: Bytes) -> Self {
    Self::Whole(Full::new(bytes))
  }

  pub(crate) fn parts(parts: Receiver<TransportPart>) -> Self {
    Self::Parts { parts, pending: None }
  }
}

impl Body for TransportBody {
  type Data = Bytes;
  type Error = Infallible;

  fn poll_frame(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
    match self.get_mut() {
      Self::Whole(whole) => Pin::new(whole).poll_frame(context),
      Self::Parts { parts, pending } => {
        if let Some(bytes) = pending.take() {
          return Poll::Ready(Some(Ok(Frame::data(bytes))));
        }

        parts.poll_recv(context).map(|part| {
          part.map(|TransportPart { header, bytes }| {
            if !bytes.is_empty() {
              *pending = Some(bytes);
            }

            Ok(Frame::data(header))
          })
        })
      }
    }
  }

  fn is_end_stream(&self) -> bool {
    match self {
      Self::Whole(whole) => whole.is_end_stream(),
      Self::Parts { .. } => false,
    }
  }

  fn size_hint(&self) -> SizeHint {
    match self {
      Self::Whole(whole) => whole.size_hint(),
      Self::Parts { .. } => SizeHint::default(),
    }
  }
}
