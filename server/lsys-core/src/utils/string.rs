use rand::seq::IndexedRandom;

pub enum RandType {
    Number,
    Upper,
    Lower,
    UpperNumber,
    LowerNumber,
    UpperHex,
    LowerHex,
}

pub fn rand_str(rand_type: RandType, len: usize) -> String {
    let base_str = match rand_type {
        RandType::Number => "0123456789",
        RandType::Upper => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        RandType::Lower => "abcdefghijklmnopqrstuvwxyz",
        RandType::UpperNumber => "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        RandType::LowerNumber => "abcdefghijklmnopqrstuvwxyz0123456789",
        RandType::UpperHex => "ABCDEF0123456789",
        RandType::LowerHex => "abcdef0123456789",
    };
    let mut rng = &mut rand::rng();
    String::from_utf8(
        base_str
            .as_bytes()
            .choose_multiple(&mut rng, len)
            .cloned()
            .collect(),
    )
    .unwrap_or_default()
}
