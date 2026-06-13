use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};

fn main() {
    let password = "000000";
    let pepper = "kkkkkkkkk".as_bytes();
    
    let salt = SaltString::generate(&mut OsRng);
    
    // 使用 m=32768, t=2, p=1 生成 Argon2id 哈希
    let params = Params::new(32768, 2, 1, None)
        .expect("Failed to create params");
    
    let argon2 = Argon2::new_with_secret(
        pepper,
        Algorithm::Argon2id,
        Version::V0x13,
        params,
    )
    .expect("Failed to create Argon2");
    
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    
    println!("Generated password hash with pepper:");
    println!("{}", hash);
    println!("\nPassword: 000000");
    println!("Pepper: kkkkkkkkk");
    println!("Parameters: m=32768, t=2, p=1");
}

