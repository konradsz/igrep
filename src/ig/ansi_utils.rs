pub fn strip_osc8_keep_text(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;

    while i < bytes.len() {
        // OSC8 start: ESC ] 8 ;
        if i + 3 < bytes.len()
            && bytes[i] == 0x1b
            && bytes[i + 1] == b']'
            && bytes[i + 2] == b'8'
            && bytes[i + 3] == b';'
        {
            // skip header
            i += 4;
            // skip until BEL (0x07) or ESC '\\' sequence
            while i < bytes.len() {
                if bytes[i] == 0x07 {
                    i += 1;
                    break;
                }
                if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // copy byte
        out.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}
