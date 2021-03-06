//! A multiplexed Chit protocol

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Framed, Encoder, Decoder};
use tokio_proto::pipeline::ServerProto;
use byteorder::{BigEndian, ByteOrder};
use std::io;
use bytes::{BytesMut};

pub type ChitRequest = Vec<u8>;
pub type  ChitResponse = Vec<u8>;

/// Our multiplexed line-based codec
pub struct  ChitCodec;

/// Protocol definition
pub struct  ChitProto;

/// Implementation of the multiplexed line-based protocol.
///
/// Frames begin with a 4 byte header, consisting of the numeric request ID
/// encoded in network order, followed by the frame payload encoded as a UTF-8
/// string and terminated with a '\n' character:
///
/// # An example frame:
///
/// +-- request id --+------- frame payload --------+
/// |                |                              |
/// | \xDEADBEEF+len | This is the frame payload    |
/// |                |                              |
/// +----------------+------------------------------+
///
impl Decoder for  ChitCodec {
    type Item =  ChitRequest;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let buf_len = buf.len();
        if buf_len < 8 {
            return Ok(None);
        }

        // check flag and msglen
        let request_id = BigEndian::read_u64(buf.as_ref());
        if request_id & 0xffffffff00000000 != 0xDEADBEEF00000000 {
            return Ok(None);
        }
        let msg_len = request_id & 0x00000000ffffffff;
        if (msg_len + 8) > buf_len as u64 {
            return Ok(None);
        }
        // ok skip the flag
        buf.split_to(8);
        // get msg
        let msg = buf.split_to(msg_len as usize);
        let mut payload = Vec::new();
        payload.extend(msg.as_ref());

        trace!("decode msg {:?} {:?}", request_id, payload);

        Ok(Some(payload.to_vec()))
    }
}

impl Encoder for  ChitCodec {
    type Item =  ChitResponse;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let request_id = 0xDEADBEEF00000000 + msg.len();
        trace!("encode msg {:?} {:?}", request_id, msg);

        let mut encoded_request_id = [0; 8];
        BigEndian::write_u64(&mut encoded_request_id, request_id as u64);

        buf.extend(&encoded_request_id);
        buf.extend(&msg);

        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for  ChitProto {
    type Request =  ChitRequest;
    type Response =  ChitResponse;

    /// `Framed<T,  ChitCodec>` is the return value of `io.framed( ChitCodec)`
    type Transport = Framed<T,  ChitCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed( ChitCodec))
    }
}
