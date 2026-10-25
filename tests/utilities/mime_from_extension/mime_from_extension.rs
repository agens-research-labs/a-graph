use crate::common::utilities::*;

/// The well-known extensions give a type, in any case; the others give none.
#[test]
fn mime_from_extension() {
    assert_eq!(mime_of("a/b.PDF").as_deref(), Some("application/pdf"));
    assert_eq!(mime_of("voice.mp3").as_deref(), Some("audio/mpeg"));
    assert_eq!(mime_of("photo.jpeg").as_deref(), Some("image/jpeg"));
    assert_eq!(mime_of("data.xyz"), None);
    assert_eq!(mime_of("noextension"), None);
}
