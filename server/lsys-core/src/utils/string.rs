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
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = base_str.as_bytes().choose(&mut rng).unwrap_or(&0);
            *idx as char
        })
        .collect()
}
