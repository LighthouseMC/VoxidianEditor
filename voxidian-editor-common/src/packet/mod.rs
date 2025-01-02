mod buf;
pub use buf::PacketBuf;
mod encode;
pub use encode::{ PacketEncode, PrefixedPacketEncode };
mod decode;
pub use decode::{ PacketDecode, PrefixedPacketDecode, DecodeError };
mod meta;
pub use meta::PacketMeta;

pub mod s2c;
pub mod c2s;


macro packet_group(
    $vis:vis enum $ident:ident { $($variantname:ident ( $variantinner:ident )),* $(,)? }
) {
    #[derive(Debug)]
    $vis enum $ident {
        $($variantname ( $variantinner )),*
    }
    impl PrefixedPacketEncode for $ident {
        fn encode_prefixed(&self, buf : &mut PacketBuf) -> () {
            match (self) {
                $(Self::$variantname(inner) => <$variantinner as PrefixedPacketEncode>::encode_prefixed(inner, buf)),*
            }
        }
    }
    impl PrefixedPacketDecode for $ident {
        fn decode_prefixed(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
            let prefix = buf.read_u8()?;
            $( if (prefix == <$variantinner as PacketMeta>::PREFIX) {
                return Ok(Self::$variantname(<$variantinner as PacketDecode>::decode(buf)?));
            } )*
            Err(DecodeError::UnknownPacketPrefix(prefix))
        }
    }
}
