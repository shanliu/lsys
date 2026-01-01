use rand::seq::{IndexedRandom, SliceRandom};

pub enum RandType {
    VaildCode,
    Number,
    Upper,
    Lower,
    UpperNumber,
    LowerNumber,
    UpperHex,
    LowerHex,
}

pub fn rand_str(rand_type: RandType, len: usize) -> String {
    let mut rng = rand::rng();

    match rand_type {
        RandType::VaildCode => {
            let base_str = "0123456789";

            // Determine same digit count based on length
            let same_count = if len >= 8 {
                len / 2 // 1/2 for length >= 8
            } else if len > 4 {
                len / 3 // 1/3 for length > 4 and < 8
            } else {
                0 // No same digits for length <= 4
            };

            if same_count > 0 {
                // Choose a random digit to repeat
                let repeated_digit = base_str.as_bytes().choose(&mut rng).unwrap_or(&b'0');

                // Generate random positions for the repeated digit
                let mut positions: Vec<usize> = (0..len).collect();
                positions.shuffle(&mut rng);
                let same_positions: std::collections::HashSet<usize> =
                    positions.iter().take(same_count).cloned().collect();

                // Generate the result string
                (0..len)
                    .map(|i| {
                        if same_positions.contains(&i) {
                            *repeated_digit as char
                        } else {
                            let idx = base_str.as_bytes().choose(&mut rng).unwrap_or(&0);
                            *idx as char
                        }
                    })
                    .collect()
            } else {
                // Normal random generation for length <= 4
                (0..len)
                    .map(|_| {
                        let idx = base_str.as_bytes().choose(&mut rng).unwrap_or(&0);
                        *idx as char
                    })
                    .collect()
            }
        }
        _ => {
            // For other types, use normal random generation
            let base_str = match rand_type {
                RandType::Number => "0123456789",
                RandType::Upper => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                RandType::Lower => "abcdefghijklmnopqrstuvwxyz",
                RandType::UpperNumber => "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
                RandType::LowerNumber => "abcdefghijklmnopqrstuvwxyz0123456789",
                RandType::UpperHex => "ABCDEF0123456789",
                RandType::LowerHex => "abcdef0123456789",
                _ => unreachable!(),
            };
            (0..len)
                .map(|_| {
                    let idx = base_str.as_bytes().choose(&mut rng).unwrap_or(&0);
                    *idx as char
                })
                .collect()
        }
    }
}
