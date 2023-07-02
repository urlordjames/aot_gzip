use flate2::{Compression, write::GzEncoder};
use tokio_util::codec::{Encoder, FramedWrite};
use bytes::{Bytes, BytesMut};
use std::io::Write;
use tokio::io::AsyncWrite;
use futures_util::Sink;

pub struct GzipEncoder {
	encoder: GzEncoder<Vec<u8>>
}

impl Encoder<Bytes> for GzipEncoder {
	type Error = std::io::Error;

	fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
		self.encoder.write_all(&item).unwrap();
		let buf = self.encoder.get_mut();
		dst.extend_from_slice(&buf);
		buf.clear();
		Ok(())
	}
}

impl Default for GzipEncoder {
	fn default() -> Self {
		Self {
			encoder: GzEncoder::new(vec![], Compression::best())
		}
	}
}

impl GzipEncoder {
	fn finish(&mut self) -> Vec<u8> {
		self.encoder.try_finish().unwrap();
		std::mem::take(&mut self.encoder.get_mut())
	}
}

use core::pin::Pin;

pub struct GzipSink<T: AsyncWrite> {
	framed_write: FramedWrite<T, GzipEncoder>
}

impl<T: AsyncWrite> GzipSink<T> {
	pub fn new(inner: T) -> Self {
		Self {
			framed_write: FramedWrite::new(inner, GzipEncoder::default())
		}
	}

	// this may be called more than once
	fn finish(&mut self) {
		let encoder = self.framed_write.encoder_mut();
		let final_buf = encoder.finish();
		let write_buffer = self.framed_write.write_buffer_mut();
		write_buffer.extend_from_slice(&final_buf);
	}
}

use core::task::{Context, Poll};

impl<T: AsyncWrite + std::marker::Unpin> Sink<Bytes> for GzipSink<T> {
	type Error = std::io::Error;

	fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.framed_write).poll_ready(cx)
	}

	fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
		Pin::new(&mut self.framed_write).start_send(item)
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.framed_write).poll_flush(cx)
	}

	fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.finish();
		Pin::new(&mut self.framed_write).poll_close(cx)
	}
}
