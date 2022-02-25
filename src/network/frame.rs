use std::io::Write;
use bytes::{Buf, BufMut, BytesMut};
use flate2::Compression;
use flate2::write::GzEncoder;
use prost::Message;
use tracing::debug;
use crate::KvError;

/// 长度占用4个字节
pub const LEN_LEN: usize = 4;

/// 长度占用31bit 所以最大的frame 是2G
const MAX_FRAME: usize = 2 * 1024 * 1024 * 1024;

const COMPRESSION_LIMIT: usize = 1436;

const COMPRESSION_BIT: usize = 1 << 31;

pub trait FrameCoder: Message + Sized + Default {
    fn encode_frame(&self, buf: &mut BytesMut) -> Result<(), KvError> {
        let size = self.encoded_len();
        if size > MAX_FRAME {
            return Err(KvError::FrameError);
        }
        buf.put_u32(size as _);

        if size > COMPRESSION_LIMIT {
            ///需要压缩
            let mut buf1 = Vec::with_capacity(size);
            self.encode(&mut buf1)?;

            let payload = buf.split_off(LEN_LEN);
            buf.clear();
            let mut encoder = GzEncoder::new(payload.writer(), Compression::default());
            encoder.write_all(&buf1[..])?;

            let payload = encoder.finish()?.into_inner();
            debug!("Encode a frame: size {}({})", size, payload.len());
            buf.put_u32((payload.len() | COMPRESSION_BIT) as _);
            buf.unsplit(payload);
            Ok(())
        } else {
            self.encode(buf)?;
            Ok(())
        }
    }

    // fn decode_frame(buf: &mut BytesMut) -> Result<Self, KvError> {
    //     let header = buf.get_u32() as usize;
    // }
}

// fn decode_header(header: usize) -> (usize, bool) {
//
// }