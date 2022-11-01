//! Decode UTF-8 characters from a stream of bytes. UTF-8 is defined
//! by the [Unicode Standard](https://home.unicode.org/).

#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use std::io::{Bytes, Read};

pub struct Utf8Decoder<R: Read> {
    bytes_iterator: Bytes<R>,
}

impl<R: Read> Utf8Decoder<R> {
    pub fn new(input: R) -> Self {
        Self {
            bytes_iterator: input.bytes(),
        }
    }
}

impl<R: Read> Iterator for Utf8Decoder<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut codepoint: u32 = 0;
        let mut bytes_remaining_count = -1;
        for byte in self.bytes_iterator.by_ref() {
            let c = byte.unwrap();
            if bytes_remaining_count == -1 {
                // read leading byte
                if c & 0x80 == 0 {
                    // 1 byte character
                    codepoint = u32::from(c);
                    return char::from_u32(codepoint);
                } else if c & 0b1110_0000 == 0b1100_0000 {
                    // 2 byte character
                    if c == 0xC0 || c == 0xC1 {
                        return Some('�');
                    }
                    codepoint = u32::from(c & 0b1_1111u8) << 6;
                    bytes_remaining_count = 1;
                } else if c & 0b1111_0000 == 0b1110_0000 {
                    // 3 byte character
                    codepoint = u32::from(c & 0b1111) << 12;
                    bytes_remaining_count = 2;
                } else if c & 0b1111_1000 == 0b1111_0000 {
                    // 4 byte character
                    codepoint = u32::from(c & 0b111) << 18;
                    bytes_remaining_count = 3;
                } else {
                    // return the replacement character in case of
                    // errors, as recommended by the standard
                    return Some('�');
                }
            } else if bytes_remaining_count > 0 {
                // continuation bytes
                if c & 0b1000_0000 == 0b1000_0000 {
                    codepoint |= u32::from(c & 0b11_1111) << (6 * (bytes_remaining_count - 1));
                    bytes_remaining_count -= 1;
                } else {
                    // return the replacement character in case of
                    // errors, as recommended by the standard
                    return Some('�');
                }

                if bytes_remaining_count == 0 {
                    // the codepoints in this range are reserved for
                    // UTF-16 surrogates
                    const SURROGATE_RANGE: std::ops::RangeInclusive<u32> = 0xD800..=0xDFFF;

                    if !SURROGATE_RANGE.contains(&codepoint) {
                        return char::from_u32(codepoint);
                    }

                    return Some('�');
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_decode_utf8_iterator() {
        let mut utf8_decoder = super::Utf8Decoder::new(&[b'a'][..]);
        assert_eq!(utf8_decoder.next(), Some('a'));
        assert_eq!(utf8_decoder.next(), None);

        let mut utf8_decoder = super::Utf8Decoder::new(&[b'a', b'\xC2', b'\xA3'][..]);
        eprintln!("£: {:#x}", '£' as u32);
        assert_eq!(utf8_decoder.next(), Some('a'));
        assert_eq!(utf8_decoder.next(), Some('£'));
        assert_eq!(utf8_decoder.next(), None);

        let mut utf8_decoder = super::Utf8Decoder::new(
            &[
                b'\xE2', b'\x82', b'\xAC', b'\xF0', b'\x90', b'\x8D', b'\x88',
            ][..],
        );
        assert_eq!(utf8_decoder.next(), Some('€'));
        assert_eq!(utf8_decoder.next(), Some('\u{10348}'));
        assert_eq!(utf8_decoder.next(), None);

        let invalid_utf8_byte: [u8; 1] = [0xff];
        let mut utf8_decoder = super::Utf8Decoder::new(&invalid_utf8_byte[..]);
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), None);
    }

    #[test]
    fn test_decode_utf8_with_utf16_surrogates() {
        // smallest high surrogate
        let mut utf8_decoder = super::Utf8Decoder::new(&[0xED, 0xA0, 0x80][..]);
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), None);

        // largest high surrogate
        let mut utf8_decoder = super::Utf8Decoder::new(&[0xED, 0xAF, 0xBF][..]);
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), None);

        //  smallest low surrogate
        let mut utf8_decoder = super::Utf8Decoder::new(&[0xED, 0xB0, 0x80][..]);
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), None);

        // largest low surrogate
        let mut utf8_decoder = super::Utf8Decoder::new(&[0xED, 0xBF, 0xBF][..]);
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), None);
    }

    #[test]
    fn test_decode_utf8_invalid_bytes() {
        let mut utf8_decoder = super::Utf8Decoder::new(&[0xC0, 0xC1][..]);
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), Some('�'));
        assert_eq!(utf8_decoder.next(), None);
    }
}
