use utf8_decode::Utf8Decoder;

#[test]
fn it_works() {
    let mut utf8_decoder = Utf8Decoder::new(&[][..]);

    utf8_decoder.next();
}
