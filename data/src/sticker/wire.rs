use super::{PackId, StickerId, StickerRef};

pub const TAG: &str = "+halloy.sticker";

pub fn encode(r: &StickerRef) -> String {
    format!("{}/{}", r.pack, r.sticker)
}

pub fn decode(value: &str) -> Option<StickerRef> {
    let (pack, sticker) = value.split_once('/')?;
    Some(StickerRef {
        pack: PackId::new(pack)?,
        sticker: StickerId::new(sticker)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(pack: &str, sticker: &str) -> StickerRef {
        StickerRef {
            pack: PackId::new(pack).unwrap(),
            sticker: StickerId::new(sticker).unwrap(),
        }
    }

    #[test]
    fn encode_basic() {
        assert_eq!(encode(&mk("dsk", "01")), "dsk/01");
        assert_eq!(encode(&mk("eyepill-2.0", "yes")), "eyepill-2.0/yes");
    }

    #[test]
    fn decode_basic() {
        assert_eq!(decode("dsk/01"), Some(mk("dsk", "01")));
        assert_eq!(
            decode("eyepill-2.0/yes"),
            Some(mk("eyepill-2.0", "yes"))
        );
    }

    #[test]
    fn roundtrip() {
        for (pack, sticker) in [("dsk", "01"), ("yes_chad", "yes"), ("a", "b")] {
            let r = mk(pack, sticker);
            assert_eq!(decode(&encode(&r)), Some(r));
        }
    }

    #[test]
    fn decode_rejects_malformed() {
        for bad in [
            "",
            "dsk",
            "/",
            "/01",
            "dsk/",
            "dsk/01/extra",
            "dsk 01",
            "d sk/01",
            "dsk/0 1",
        ] {
            assert_eq!(decode(bad), None, "should reject: {bad:?}");
        }
    }

    #[test]
    fn tag_constant() {
        assert_eq!(TAG, "+halloy.sticker");
    }
}
