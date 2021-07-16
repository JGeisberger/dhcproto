use std::net::Ipv4Addr;

mod flags;
mod htype;
mod opcode;
mod options;

// re-export submodules from proto::msg
pub use self::{flags::*, htype::*, opcode::*, options::*};

use crate::{
    decoder::{Decodable, Decoder},
    encoder::{Encodable, Encoder},
    error::*,
};

const MAGIC: [u8; 4] = [99, 130, 83, 99];

/// [Dynamic Host Configuration Protocol](https://tools.ietf.org/html/rfc2131#section-2)
///
///```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |     op (1)    |   htype (1)   |   hlen (1)    |   hops (1)    |
/// +---------------+---------------+---------------+---------------+
/// |                            xid (4)                            |
/// +-------------------------------+-------------------------------+
/// |           secs (2)            |           flags (2)           |
/// +-------------------------------+-------------------------------+
/// |                          ciaddr  (4)                          |
/// +---------------------------------------------------------------+
/// |                          yiaddr  (4)                          |
/// +---------------------------------------------------------------+
/// |                          siaddr  (4)                          |
/// +---------------------------------------------------------------+
/// |                          giaddr  (4)                          |
/// +---------------------------------------------------------------+
/// |                          chaddr  (16)                         |
/// +---------------------------------------------------------------+
/// |                          sname   (64)                         |
/// +---------------------------------------------------------------+
/// |                          file    (128)                        |
/// +---------------------------------------------------------------+
/// |                          options (variable)                   |
/// +---------------------------------------------------------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// op code / message type
    opcode: Opcode,
    /// Hardware address type: https://tools.ietf.org/html/rfc3232
    htype: HType,
    /// Hardware address length
    hlen: u8,
    /// Client sets to zero, optionally used by relay agents when booting via a relay agent.
    hops: u8,
    /// Transaction ID, a random number chosen by the client
    xid: u32,
    /// seconds elapsed since client began address acquisition or renewal process
    secs: u16,
    /// Flags
    flags: Flags,
    /// Client IP
    ciaddr: Ipv4Addr,
    /// Your IP
    yiaddr: Ipv4Addr,
    /// Server IP
    siaddr: Ipv4Addr,
    /// Gateway IP
    giaddr: Ipv4Addr,
    /// Client hardware address
    chaddr: [u8; 16],
    /// Server hostname
    sname: Option<String>,
    // File name
    file: Option<String>,
    magic: [u8; 4],
    opts: DhcpOptions,
}

impl Message {
    /// Get the message's opcode.
    pub fn opcode(&self) -> Opcode {
        self.opcode
    }

    /// Get the message's htype.
    pub fn htype(&self) -> &HType {
        &self.htype
    }

    /// Get the message's hlen.
    pub fn hlen(&self) -> u8 {
        self.hlen
    }

    /// Get the message's hops.
    pub fn hops(&self) -> u8 {
        self.hops
    }

    /// Get the message's chaddr.
    pub fn chaddr(&self) -> [u8; 16] {
        self.chaddr
    }

    /// Get the message's giaddr.
    pub fn giaddr(&self) -> Ipv4Addr {
        self.giaddr
    }

    /// Get the message's siaddr.
    pub fn siaddr(&self) -> Ipv4Addr {
        self.siaddr
    }

    /// Get the message's yiaddr.
    pub fn yiaddr(&self) -> Ipv4Addr {
        self.yiaddr
    }

    /// Get the message's ciaddr.
    pub fn ciaddr(&self) -> Ipv4Addr {
        self.ciaddr
    }

    /// Get the message's flags.
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Get the message's secs.
    pub fn secs(&self) -> u16 {
        self.secs
    }

    /// Get the message's xid.
    pub fn xid(&self) -> u32 {
        self.xid
    }

    /// Get a reference to the message's file.
    pub fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }

    /// Get a reference to the message's sname.
    pub fn sname(&self) -> Option<&String> {
        self.sname.as_ref()
    }

    /// Get a reference to the message's opts.
    pub fn opts(&self) -> &DhcpOptions {
        &self.opts
    }

    /// Get a mutable reference to the message's options.
    pub fn opts_mut(&mut self) -> &mut DhcpOptions {
        &mut self.opts
    }
}

impl<'r> Decodable<'r> for Message {
    fn decode(decoder: &mut Decoder<'r>) -> DecodeResult<Self> {
        Ok(Message {
            opcode: Opcode::decode(decoder)?,
            htype: decoder.read_u8()?.into(),
            hlen: decoder.read_u8()?,
            hops: decoder.read_u8()?,
            xid: decoder.read_u32()?,
            secs: decoder.read_u16()?,
            flags: decoder.read_u16()?.into(),
            ciaddr: decoder.read_u32()?.into(),
            yiaddr: decoder.read_u32()?.into(),
            siaddr: decoder.read_u32()?.into(),
            giaddr: decoder.read_u32()?.into(),
            chaddr: decoder.read::<16>()?,
            sname: decoder.read_const_string::<64>()?,
            file: decoder.read_const_string::<128>()?,
            // TODO: check magic bytes against expected?
            magic: decoder.read::<4>()?,
            opts: DhcpOptions::decode(decoder)?,
        })
    }
}

impl<'a> Encodable<'a> for Message {
    fn encode(&self, e: &'_ mut Encoder<'a>) -> EncodeResult<()> {
        self.opcode.encode(e)?;
        self.htype.encode(e)?;
        e.write_u8(self.hlen)?;
        e.write_u8(self.hops)?;
        e.write_u32(self.xid)?;
        e.write_u16(self.secs)?;
        e.write_u16(self.flags.into())?;
        e.write_u32(self.ciaddr.into())?;
        e.write_u32(self.yiaddr.into())?;
        e.write_u32(self.siaddr.into())?;
        e.write_u32(self.giaddr.into())?;
        e.write_slice(&self.chaddr[..])?;
        e.write_fill_string(&self.sname, 64)?;
        e.write_fill_string(&self.file, 128)?;

        e.write(self.magic)?;
        self.opts.encode(e)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn decode_offer() -> Result<()> {
        // decode
        let offer = dhcp_offer();
        let msg = Message::decode(&mut Decoder::new(&offer))?;
        dbg!(&msg);
        dbg!(offer.len());
        // now encode
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        msg.encode(&mut e)?;
        println!("{:?}", &buf);
        println!("{:?}", &dhcp_offer());
        // len will be different because input has arbitrary PAD bytes
        // assert_eq!(buf.len(), dhcp_offer().len());
        // decode again
        let res = Message::decode(&mut Decoder::new(&buf))?;
        // check Messages are equal after decoding/encoding
        assert_eq!(msg, res);
        Ok(())
    }

    #[test]
    fn decode_bootreq() -> Result<()> {
        println!("{:02x?}", &MAGIC[..]);
        let offer = dhcp_bootreq();
        let msg = Message::decode(&mut Decoder::new(&offer))?;
        // now encode
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        msg.encode(&mut e)?;
        assert_eq!(buf, dhcp_bootreq());
        Ok(())
    }

    fn dhcp_offer() -> Vec<u8> {
        vec![
            0x02, 0x01, 0x06, 0x00, 0x00, 0x00, 0x15, 0x5c, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xc0, 0xa8, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xcc, 0x00, 0x0a, 0xc4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x82,
            0x53, 0x63, 0x35, 0x01, 0x02, 0x36, 0x04, 0xc0, 0xa8, 0x00, 0x01, 0x33, 0x04, 0x00,
            0x00, 0x00, 0x3c, 0x3a, 0x04, 0x00, 0x00, 0x00, 0x1e, 0x3b, 0x04, 0x00, 0x00, 0x00,
            0x34, 0x01, 0x04, 0xff, 0xff, 0xff, 0x00, 0x03, 0x04, 0xc0, 0xa8, 0x00, 0x01, 0x06,
            0x08, 0xc0, 0xa8, 0x00, 0x01, 0xc0, 0xa8, 0x01, 0x01, 0xff, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }
    fn dhcp_bootreq() -> Vec<u8> {
        vec![
            1u8, // op
            2,   // htype
            3,   // hlen
            4,   // ops
            5, 6, 7, 8, // xid
            9, 10, // secs
            11, 12, // flags
            13, 14, 15, 16, // ciaddr
            17, 18, 19, 20, // yiaddr
            21, 22, 23, 24, // siaddr
            25, 26, 27, 28, // giaddr
            29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, // chaddr
            45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66,
            67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
            89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107,
            0, // sname: "-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijk",
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            109, 0, 0, 0, 0, 0, 0, 0,
            0, // file: "mnopqrstuvwxyz{|}mnopqrstuvwxyz{|}mnopqrstuvwxyz{|}mnopqrstuvwxyz{|}mnopqrstuvwxyz{|}mnopqrstuvwxyz{|}mnopqrstuvwxyz{|}m",
            99, 130, 83, 99, // magic cookie
        ]
    }
}
