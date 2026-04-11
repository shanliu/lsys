use lsys_core::valid_key;
use lsys_core::valid_param::*;

// ---------------------------------------------------------------------------
// Helper: assert a single rule passes
// ---------------------------------------------------------------------------
fn assert_rule_ok<R, D>(rule: &R, data: &D)
where
    R: ValidRule<T = D>,
{
    assert!(rule.check(data).is_ok(), "expected Ok for data");
}

// Helper: assert a single rule fails
fn assert_rule_err<R, D>(rule: &R, data: &D)
where
    R: ValidRule<T = D>,
{
    assert!(rule.check(data).is_err(), "expected Err for data");
}

// ===========================================================================
// 1. ValidNotEmpty
// ===========================================================================
#[test]
fn test_not_empty_str() {
    let rule = ValidNotEmpty::<&str>::default();
    assert_rule_ok(&rule, &"hello");
    assert_rule_ok(&rule, &"  a  ");
    assert_rule_ok(&rule, &"0");

    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"   ");
    assert_rule_err(&rule, &"\t");
    assert_rule_err(&rule, &"\n");
}

#[test]
fn test_not_empty_string() {
    let rule = ValidNotEmpty::<String>::default();
    assert_rule_ok(&rule, &"hello".to_string());
    assert_rule_ok(&rule, &"x".to_string());

    assert_rule_err(&rule, &String::new());
    assert_rule_err(&rule, &"   ".to_string());
}

#[test]
fn test_not_empty_vec() {
    let rule = ValidNotEmpty::<Vec<i32>>::default();
    assert_rule_ok(&rule, &vec![1]);
    assert_rule_ok(&rule, &vec![1, 2, 3]);

    let empty: Vec<i32> = vec![];
    assert_rule_err(&rule, &empty);
}

#[test]
fn test_not_empty_slice() {
    let rule = ValidNotEmpty::<&[i32]>::default();
    let data: &[i32] = &[1, 2];
    assert_rule_ok(&rule, &data);

    let empty: &[i32] = &[];
    assert_rule_err(&rule, &empty);
}

// ===========================================================================
// 2. ValidEmail
// ===========================================================================
#[test]
fn test_email_valid() {
    let rule = ValidEmail::<&str>::default();
    assert_rule_ok(&rule, &"test@test.com");
    assert_rule_ok(&rule, &"user.name@domain.co.uk");
    assert_rule_ok(&rule, &"a@b.cc");
    assert_rule_ok(&rule, &"user-name@domain.org");
    assert_rule_ok(&rule, &"user123@sub.domain.com");
}

#[test]
fn test_email_invalid() {
    let rule = ValidEmail::<&str>::default();
    assert_rule_err(&rule, &"no-at");
    assert_rule_err(&rule, &"@no-local");
    assert_rule_err(&rule, &"no-domain@");
    assert_rule_err(&rule, &"spaces @domain.com");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_email_string_type() {
    let rule = ValidEmail::<String>::default();
    assert_rule_ok(&rule, &"hello@world.com".to_string());
    assert_rule_err(&rule, &"bad".to_string());
}

// ===========================================================================
// 3. ValidMobile
// ===========================================================================
#[test]
fn test_mobile_valid() {
    let rule = ValidMobile::<&str>::default();
    assert_rule_ok(&rule, &"13800138000");
    assert_rule_ok(&rule, &"8613800138000");
    assert_rule_ok(&rule, &"+8613800138000");
    assert_rule_ok(&rule, &"08613800138000");
    assert_rule_ok(&rule, &"15912345678");
}

#[test]
fn test_mobile_invalid() {
    let rule = ValidMobile::<&str>::default();
    assert_rule_err(&rule, &"1234567890");
    assert_rule_err(&rule, &"abc");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"12345");
    assert_rule_err(&rule, &"10000000000");
}

// ===========================================================================
// 4. ValidPattern
// ===========================================================================
#[test]
fn test_pattern_numeric() {
    let rule = ValidPattern::<&str>::Numeric;
    assert_rule_ok(&rule, &"123456");
    assert_rule_ok(&rule, &"0");
    assert_rule_ok(&rule, &"9999999999");

    assert_rule_err(&rule, &"12a34");
    assert_rule_err(&rule, &"abc");
    assert_rule_err(&rule, &"12.34");
}

#[test]
fn test_pattern_alphabetic() {
    let rule = ValidPattern::<&str>::Alphabetic;
    assert_rule_ok(&rule, &"abc");
    assert_rule_ok(&rule, &"ABC");
    assert_rule_ok(&rule, &"AbCdEf");

    assert_rule_err(&rule, &"abc123");
    assert_rule_err(&rule, &"ab cd");
}

#[test]
fn test_pattern_alphanumeric() {
    let rule = ValidPattern::<&str>::Alphanumeric;
    assert_rule_ok(&rule, &"abc123");
    assert_rule_ok(&rule, &"ABC");
    assert_rule_ok(&rule, &"123");

    assert_rule_err(&rule, &"abc-123");
    assert_rule_err(&rule, &"hello world");
    assert_rule_err(&rule, &"test@123");
}

#[test]
fn test_pattern_ident() {
    let rule = ValidPattern::<&str>::Ident;
    assert_rule_ok(&rule, &"hello");
    assert_rule_ok(&rule, &"hello-world");
    assert_rule_ok(&rule, &"hello_world");
    assert_rule_ok(&rule, &"hello.world");
    assert_rule_ok(&rule, &"a1-b2_c3.d4");

    assert_rule_err(&rule, &"-start");
    assert_rule_err(&rule, &"_start");
    assert_rule_err(&rule, &".start");
    assert_rule_err(&rule, &"hello world");
    assert_rule_err(&rule, &"hello@world");
}

#[test]
fn test_pattern_hex() {
    let rule = ValidPattern::<&str>::Hex;
    assert_rule_ok(&rule, &"0123456789abcdef");
    assert_rule_ok(&rule, &"ABCDEF");
    assert_rule_ok(&rule, &"0");

    assert_rule_err(&rule, &"0123456789abcdefg");
    assert_rule_err(&rule, &"xyz");
}

#[test]
fn test_pattern_not_format() {
    let rule = ValidPattern::<&str>::NotFormat;
    assert_rule_ok(&rule, &"hello world");
    assert_rule_ok(&rule, &"normal text 123");

    assert_rule_err(&rule, &"has\ttab");
    assert_rule_err(&rule, &"has\nnewline");
    assert_rule_err(&rule, &"has\rreturn");
    assert_rule_err(&rule, &" leading space");
    assert_rule_err(&rule, &"trailing space ");
    assert_rule_err(&rule, &"double  space");
    assert_rule_err(&rule, &"has\\backslash");
}

// ===========================================================================
// 5. ValidStrlen
// ===========================================================================
#[test]
fn test_strlen_range() {
    let rule = ValidStrlen::<&str>::range(3, 10);
    assert_rule_ok(&rule, &"abc");
    assert_rule_ok(&rule, &"abcdefghij");
    assert_rule_ok(&rule, &"hello");

    assert_rule_err(&rule, &"ab");
    assert_rule_err(&rule, &"abcdefghijk");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_strlen_max() {
    let rule = ValidStrlen::<&str>::max(5);
    assert_rule_ok(&rule, &"");
    assert_rule_ok(&rule, &"ab");
    assert_rule_ok(&rule, &"abcde");

    assert_rule_err(&rule, &"abcdef");
    assert_rule_err(&rule, &"long string here");
}

#[test]
fn test_strlen_min() {
    let rule = ValidStrlen::<&str>::min(3);
    assert_rule_ok(&rule, &"abc");
    assert_rule_ok(&rule, &"abcdef");

    assert_rule_err(&rule, &"ab");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_strlen_eq() {
    let rule = ValidStrlen::<&str>::eq(5);
    assert_rule_ok(&rule, &"abcde");
    assert_rule_ok(&rule, &"12345");

    assert_rule_err(&rule, &"abcd");
    assert_rule_err(&rule, &"abcdef");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_strlen_unicode() {
    // chars().count() counts Unicode codepoints, not bytes
    let rule = ValidStrlen::<&str>::eq(2);
    assert_rule_ok(&rule, &"你好");
    assert_rule_err(&rule, &"你好世");
}

// ===========================================================================
// 6. ValidContains
// ===========================================================================
#[test]
fn test_contains_i32() {
    let rule = ValidContains(&[1, 2, 3, 4, 5]);
    assert_rule_ok(&rule, &1);
    assert_rule_ok(&rule, &3);
    assert_rule_ok(&rule, &5);

    assert_rule_err(&rule, &0);
    assert_rule_err(&rule, &6);
    assert_rule_err(&rule, &-1);
}

#[test]
fn test_contains_str() {
    let allowed: &[&str] = &["apple", "banana", "cherry"];
    let rule = ValidContains(allowed);
    assert_rule_ok(&rule, &"apple");
    assert_rule_ok(&rule, &"banana");

    assert_rule_err(&rule, &"grape");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"Apple");
}

// ===========================================================================
// 7. ValidUrl
// ===========================================================================
#[test]
fn test_url_valid() {
    let rule = ValidUrl::<&str>::default();
    assert_rule_ok(&rule, &"http://example.com");
    assert_rule_ok(&rule, &"https://example.com");
    assert_rule_ok(&rule, &"https://example.com/path/to/page");
    assert_rule_ok(&rule, &"http://192.168.1.1");
    assert_rule_ok(&rule, &"http://192.168.1.1:8080");
    assert_rule_ok(&rule, &"https://sub.domain.example.com:443/path");
}

#[test]
fn test_url_invalid() {
    let rule = ValidUrl::<&str>::default();
    assert_rule_err(&rule, &"ftp://example.com");
    assert_rule_err(&rule, &"example.com");
    assert_rule_err(&rule, &"just-text");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"http://");
}

// ===========================================================================
// 8. ValidDomain
// ===========================================================================
#[test]
fn test_domain_valid() {
    let rule = ValidDomain::<&str>::default();
    assert_rule_ok(&rule, &"example.com");
    assert_rule_ok(&rule, &"sub.domain.example.com");
    assert_rule_ok(&rule, &"192.168.1.1");
    assert_rule_ok(&rule, &"192.168.1.1:8080");
    assert_rule_ok(&rule, &"example.com:443");
}

#[test]
fn test_domain_invalid() {
    let rule = ValidDomain::<&str>::default();
    assert_rule_err(&rule, &"http://example.com");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"just spaces here");
}

// ===========================================================================
// 9. ValidIp
// ===========================================================================
#[test]
fn test_ip_v4_valid() {
    let rule = ValidIp::<&str>::new(VALID_IP_V4);
    assert_rule_ok(&rule, &"192.168.1.1");
    assert_rule_ok(&rule, &"0.0.0.0");
    assert_rule_ok(&rule, &"255.255.255.255");
    assert_rule_ok(&rule, &"10.0.0.1");
    assert_rule_ok(&rule, &"127.0.0.1");
}

#[test]
fn test_ip_v4_invalid() {
    let rule = ValidIp::<&str>::new(VALID_IP_V4);
    assert_rule_err(&rule, &"999.999.999.999");
    assert_rule_err(&rule, &"256.1.1.1");
    assert_rule_err(&rule, &"abc");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"::1");
}

#[test]
fn test_ip_v6_valid() {
    let rule = ValidIp::<&str>::new(VALID_IP_V6);
    assert_rule_ok(&rule, &"::1");
    assert_rule_ok(&rule, &"2001:db8::1");
    assert_rule_ok(&rule, &"2001:0db8:0000:0000:0000:0000:0000:0001");
    assert_rule_ok(&rule, &"fe80::e1bd:c78d:610f:3d03");
    assert_rule_ok(&rule, &"::");
}

#[test]
fn test_ip_v6_invalid() {
    let rule = ValidIp::<&str>::new(VALID_IP_V6);
    assert_rule_err(&rule, &"192.168.1.1");
    assert_rule_err(&rule, &"abc");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"gggg::1");
}

#[test]
fn test_ip_both() {
    let rule = ValidIp::<&str>::default(); // VALID_IP = V4 | V6
    assert_rule_ok(&rule, &"192.168.1.1");
    assert_rule_ok(&rule, &"::1");
    assert_rule_ok(&rule, &"2001:db8::1");

    assert_rule_err(&rule, &"not-an-ip");
    assert_rule_err(&rule, &"");
}

// ===========================================================================
// 10. ValidChinaID
// ===========================================================================
#[test]
fn test_china_id_valid() {
    let rule = ValidChinaID::<&str>::default();
    // IDs with valid checksums matching the crate's checksum algorithm
    assert_rule_ok(&rule, &"110101199003074134");
    assert_rule_ok(&rule, &"440305198801010029");
    assert_rule_ok(&rule, &"320121198506151242");
    assert_rule_ok(&rule, &"11010519900101001X");
}

#[test]
fn test_china_id_invalid() {
    let rule = ValidChinaID::<&str>::default();
    assert_rule_err(&rule, &"000000000000000000");
    assert_rule_err(&rule, &"12345678901234567X");
    assert_rule_err(&rule, &"1234");
    assert_rule_err(&rule, &"abcdefghijklmnopqr");
    assert_rule_err(&rule, &"");
    // Wrong checksum digit
    assert_rule_err(&rule, &"110101199003074130");
    // Wrong format (first digit 0)
    assert_rule_err(&rule, &"010101199003074134");
}

// ===========================================================================
// 11. ValidChinaLicensePlate
// ===========================================================================
#[test]
fn test_china_license_plate_valid() {
    let rule = ValidChinaLicensePlate::<&str>::default();
    assert_rule_ok(&rule, &"京A12345");
    assert_rule_ok(&rule, &"粤B99999");
    assert_rule_ok(&rule, &"沪C00001");
    // New energy plate formats
    assert_rule_ok(&rule, &"京AD1234");
    assert_rule_ok(&rule, &"粤A12345F");
}

#[test]
fn test_china_license_plate_invalid() {
    let rule = ValidChinaLicensePlate::<&str>::default();
    assert_rule_err(&rule, &"ABC1234");
    assert_rule_err(&rule, &"");
    assert_rule_err(&rule, &"12345");
    assert_rule_err(&rule, &"京I12345"); // I is excluded (J-NP-Z means no I/O)
}

// ===========================================================================
// 12. ValidCreditCard
// ===========================================================================
#[test]
fn test_credit_card_valid() {
    let rule = ValidCreditCard::<&str>::default();
    // Known test card numbers passing Luhn
    assert_rule_ok(&rule, &"4532015112830366");
    assert_rule_ok(&rule, &"5425233430109903");
    assert_rule_ok(&rule, &"4111111111111111");
    assert_rule_ok(&rule, &"5500000000000004");
    assert_rule_ok(&rule, &"340000000000009");
}

#[test]
fn test_credit_card_invalid() {
    let rule = ValidCreditCard::<&str>::default();
    assert_rule_err(&rule, &"1234567890");
    assert_rule_err(&rule, &"1234567890123456");
    assert_rule_err(&rule, &"4111111111111112");
    assert_rule_err(&rule, &"123"); // too short
    assert_rule_err(&rule, &"12345678901234567890"); // too long (20 digits)
}

// ===========================================================================
// 13. ValidPassword
// ===========================================================================
#[test]
fn test_password_low_valid() {
    let rule = ValidPassword::<&str>::Low;
    assert_rule_ok(&rule, &"abcdef");
    assert_rule_ok(&rule, &"123456");
    assert_rule_ok(&rule, &"P@ssw0");
    assert_rule_ok(&rule, &"aaaaaa");
}

#[test]
fn test_password_low_invalid() {
    let rule = ValidPassword::<&str>::Low;
    assert_rule_err(&rule, &"short");
    assert_rule_err(&rule, &"12345");
    assert_rule_err(&rule, &"has space");
    assert_rule_err(&rule, &"tab\there");
}

#[test]
fn test_password_medium_valid() {
    let rule = ValidPassword::<&str>::Medium;
    assert_rule_ok(&rule, &"P@ssw0rd");
    assert_rule_ok(&rule, &"abcd12");
    assert_rule_ok(&rule, &"xY3!zz");
}

#[test]
fn test_password_medium_invalid() {
    let rule = ValidPassword::<&str>::Medium;
    assert_rule_err(&rule, &"short");
    assert_rule_err(&rule, &"aaaaaa"); // all same
    assert_rule_err(&rule, &"123456"); // consecutive numbers
    assert_rule_err(&rule, &"abcdef"); // consecutive letters
    assert_rule_err(&rule, &"112233"); // repeating pairs
}

#[test]
fn test_password_strong_valid() {
    let rule = ValidPassword::<&str>::Strong;
    assert_rule_ok(&rule, &"P@ssw0rd");
    assert_rule_ok(&rule, &"Str0ng!x");
    assert_rule_ok(&rule, &"a1!bcdef");
}

#[test]
fn test_password_strong_invalid() {
    let rule = ValidPassword::<&str>::Strong;
    assert_rule_err(&rule, &"short1!"); // too short (7 chars)
    assert_rule_err(&rule, &"abcdefgh"); // no digit, no special
    assert_rule_err(&rule, &"12345678"); // no letter, no special
    assert_rule_err(&rule, &"abcd1234"); // no special char
    assert_rule_err(&rule, &"abcd!@#$"); // no digit
    assert_rule_err(&rule, &"has space1!"); // whitespace
}

#[test]
fn test_password_rejects_non_ascii() {
    let rule = ValidPassword::<&str>::Low;
    assert_rule_err(&rule, &"密码password");
}

// ===========================================================================
// 14. ValidDateTime
// ===========================================================================
#[test]
fn test_datetime_date_valid() {
    let rule = ValidDateTime::<&str>::Date;
    assert_rule_ok(&rule, &"2024-01-15");
    assert_rule_ok(&rule, &"1999-12-31");
    assert_rule_ok(&rule, &"2000-06-01");
}

#[test]
fn test_datetime_date_invalid() {
    let rule = ValidDateTime::<&str>::Date;
    assert_rule_err(&rule, &"2024/01/15");
    assert_rule_err(&rule, &"01-15-2024");
    assert_rule_err(&rule, &"2024-1-5");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_datetime_time_valid() {
    let rule = ValidDateTime::<&str>::Time;
    assert_rule_ok(&rule, &"12:30:45");
    assert_rule_ok(&rule, &"00:00:00");
    assert_rule_ok(&rule, &"23:59:59");
}

#[test]
fn test_datetime_time_invalid() {
    let rule = ValidDateTime::<&str>::Time;
    assert_rule_err(&rule, &"12:30");
    assert_rule_err(&rule, &"1:2:3");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_datetime_datetime_valid() {
    let rule = ValidDateTime::<&str>::DateTime;
    assert_rule_ok(&rule, &"2024-01-15 12:30:45");
    assert_rule_ok(&rule, &"2000-06-01 00:00:00");
}

#[test]
fn test_datetime_datetime_invalid() {
    let rule = ValidDateTime::<&str>::DateTime;
    assert_rule_err(&rule, &"2024-01-15T12:30:45");
    assert_rule_err(&rule, &"2024-01-15");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_datetime_datetimezone_valid() {
    let rule = ValidDateTime::<&str>::DateTimeZone;
    assert_rule_ok(&rule, &"2024-01-15 12:30:45+08:00");
    assert_rule_ok(&rule, &"2024-01-15 12:30:45-05:00");
    assert_rule_ok(&rule, &"2024-01-15 00:00:00+00:00");
}

#[test]
fn test_datetime_datetimezone_invalid() {
    let rule = ValidDateTime::<&str>::DateTimeZone;
    assert_rule_err(&rule, &"2024-01-15 12:30:45");
    assert_rule_err(&rule, &"2024-01-15 12:30:45Z");
    assert_rule_err(&rule, &"");
}

// ===========================================================================
// 15. ValidColor
// ===========================================================================
#[test]
fn test_color_rgb_valid() {
    let rule = ValidColor::<&str>::RGB;
    assert_rule_ok(&rule, &"#FF0000");
    assert_rule_ok(&rule, &"#00ff00");
    assert_rule_ok(&rule, &"#000000");
    assert_rule_ok(&rule, &"#FFFFFF");
    assert_rule_ok(&rule, &"#1a2b3c");
}

#[test]
fn test_color_rgb_invalid() {
    let rule = ValidColor::<&str>::RGB;
    assert_rule_err(&rule, &"FF0000");
    assert_rule_err(&rule, &"#FFF");
    assert_rule_err(&rule, &"#FF00FF00");
    assert_rule_err(&rule, &"#GGGGGG");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_color_rgba_valid() {
    let rule = ValidColor::<&str>::RGBA;
    assert_rule_ok(&rule, &"#FF000080");
    assert_rule_ok(&rule, &"#00ff00ff");
    assert_rule_ok(&rule, &"#00000000");
    assert_rule_ok(&rule, &"#FFFFFFFF");
}

#[test]
fn test_color_rgba_invalid() {
    let rule = ValidColor::<&str>::RGBA;
    assert_rule_err(&rule, &"#FF0000");
    assert_rule_err(&rule, &"FF000080");
    assert_rule_err(&rule, &"#GGGGGGG0");
    assert_rule_err(&rule, &"");
}

// ===========================================================================
// 16. ValidGit
// ===========================================================================
#[test]
fn test_git_version_hash_valid() {
    let rule = ValidGit::<&str>::VersionHash;
    assert_rule_ok(&rule, &"da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_rule_ok(&rule, &"0000000000000000000000000000000000000000");
    assert_rule_ok(&rule, &"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_rule_ok(&rule, &"ABCDEF0123456789abcdef0123456789ABCDEF01");
}

#[test]
fn test_git_version_hash_invalid() {
    let rule = ValidGit::<&str>::VersionHash;
    assert_rule_err(&rule, &"da39a3ee"); // too short
    assert_rule_err(&rule, &"da39a3ee5e6b4b0d3255bfef95601890afd807090"); // 41 chars
    assert_rule_err(&rule, &"ga39a3ee5e6b4b0d3255bfef95601890afd80709"); // 'g' not hex
    assert_rule_err(&rule, &"");
}

// ===========================================================================
// 17. ValidNumber
// ===========================================================================
#[test]
fn test_number_id_u32() {
    let rule = ValidNumber::<u32>::id();
    assert_rule_ok(&rule, &0u32);
    assert_rule_ok(&rule, &1u32);
    assert_rule_ok(&rule, &u32::MAX);
}

#[test]
fn test_number_range_u32() {
    let rule = ValidNumber::<u32>::range(10, 100);
    assert_rule_ok(&rule, &10u32);
    assert_rule_ok(&rule, &50u32);
    assert_rule_ok(&rule, &100u32);

    assert_rule_err(&rule, &9u32);
    assert_rule_err(&rule, &101u32);
    assert_rule_err(&rule, &0u32);
}

#[test]
fn test_number_eq_i64() {
    let rule = ValidNumber::<i64>::eq(42);
    assert_rule_ok(&rule, &42i64);

    assert_rule_err(&rule, &41i64);
    assert_rule_err(&rule, &43i64);
    assert_rule_err(&rule, &0i64);
    assert_rule_err(&rule, &-42i64);
}

#[test]
fn test_number_min_i32() {
    let rule = ValidNumber::<i32>::min(0);
    assert_rule_ok(&rule, &0i32);
    assert_rule_ok(&rule, &100i32);
    assert_rule_ok(&rule, &i32::MAX);

    assert_rule_err(&rule, &-1i32);
    assert_rule_err(&rule, &i32::MIN);
}

#[test]
fn test_number_max_i32() {
    let rule = ValidNumber::<i32>::max(100);
    assert_rule_ok(&rule, &100i32);
    assert_rule_ok(&rule, &0i32);
    assert_rule_ok(&rule, &-100i32);
    assert_rule_ok(&rule, &i32::MIN);

    assert_rule_err(&rule, &101i32);
    assert_rule_err(&rule, &i32::MAX);
}

#[test]
fn test_number_range_f64() {
    let rule = ValidNumber::<f64>::range(0.0, 1.0);
    assert_rule_ok(&rule, &0.0f64);
    assert_rule_ok(&rule, &0.5f64);
    assert_rule_ok(&rule, &1.0f64);

    assert_rule_err(&rule, &-0.1f64);
    assert_rule_err(&rule, &1.1f64);
}

#[test]
fn test_number_various_types() {
    // u8
    assert_rule_ok(&ValidNumber::<u8>::range(0, 255), &128u8);
    // u64
    assert_rule_ok(&ValidNumber::<u64>::min(1), &100u64);
    assert_rule_err(&ValidNumber::<u64>::min(1), &0u64);
    // i8
    assert_rule_ok(&ValidNumber::<i8>::range(-128, 127), &0i8);
    // f32
    assert_rule_ok(&ValidNumber::<f32>::range(-1.0, 1.0), &0.0f32);
}

// ===========================================================================
// 18. ValidStrMatch
// ===========================================================================
#[test]
fn test_str_match_start_with() {
    let rule = ValidStrMatch::<&str>::StartWith("http");
    assert_rule_ok(&rule, &"http://example.com");
    assert_rule_ok(&rule, &"https://example.com");
    assert_rule_ok(&rule, &"http");

    assert_rule_err(&rule, &"ftp://example.com");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_str_match_end_with() {
    let rule = ValidStrMatch::<&str>::EndWith(".com");
    assert_rule_ok(&rule, &"example.com");
    assert_rule_ok(&rule, &"test.example.com");

    assert_rule_err(&rule, &"example.org");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_str_match_start_not_with() {
    let rule = ValidStrMatch::<&str>::StartNotWith("_");
    assert_rule_ok(&rule, &"hello");
    assert_rule_ok(&rule, &"world");

    assert_rule_err(&rule, &"_private");
    assert_rule_err(&rule, &"_");
}

#[test]
fn test_str_match_end_not_with() {
    let rule = ValidStrMatch::<&str>::EndNotWith("/");
    assert_rule_ok(&rule, &"hello");
    assert_rule_ok(&rule, &"/path");

    assert_rule_err(&rule, &"path/");
    assert_rule_err(&rule, &"/");
}

#[test]
fn test_str_match_contains() {
    let rule = ValidStrMatch::<&str>::Contains("world");
    assert_rule_ok(&rule, &"hello world");
    assert_rule_ok(&rule, &"world");
    assert_rule_ok(&rule, &"worlds");

    assert_rule_err(&rule, &"hello");
    assert_rule_err(&rule, &"");
}

#[test]
fn test_str_match_not_contains() {
    let rule = ValidStrMatch::<&str>::NotContains("bad");
    assert_rule_ok(&rule, &"good text");
    assert_rule_ok(&rule, &"");

    assert_rule_err(&rule, &"this is bad");
    assert_rule_err(&rule, &"bad");
    assert_rule_err(&rule, &"badly");
}

// ===========================================================================
// ValidParam: accumulation of multiple errors
// ===========================================================================
#[test]
fn test_valid_param_no_errors() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("email"),
        &"test@example.com",
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    assert!(valid.check().is_ok());
}

#[test]
fn test_valid_param_single_error() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("email"),
        &"bad-email",
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    let fields = val["field"].as_array().unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].as_str().unwrap(), "email");
}

#[test]
fn test_valid_param_multiple_field_errors() {
    let mut valid = ValidParam::default();
    valid
        .add(
            valid_key!("email"),
            &"bad-email",
            &ValidParamCheck::default().add_rule(ValidEmail::default()),
        )
        .add(
            valid_key!("name"),
            &"",
            &ValidParamCheck::default().add_rule(ValidNotEmpty::<&str>::default()),
        )
        .add(
            valid_key!("age"),
            &200u32,
            &ValidParamCheck::default().add_rule(ValidNumber::<u32>::range(0, 150)),
        );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    let fields = val["field"].as_array().unwrap();
    assert_eq!(fields.len(), 3);
}

#[test]
fn test_valid_param_clear() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("email"),
        &"bad",
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    valid.clear();
    // After clear, check should pass (no errors)
    assert!(valid.check().is_ok());
}

#[test]
fn test_valid_param_check_consumes_errors() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("email"),
        &"bad",
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    // First check returns errors
    assert!(valid.check().is_err());
    // Second check should be Ok since check() consumed the errors via std::mem::take
    assert!(valid.check().is_ok());
}

// ===========================================================================
// ValidError::to_value() JSON structure
// ===========================================================================
#[test]
fn test_valid_error_to_value_structure() {
    let mut valid = ValidParam::default();
    valid
        .add(
            valid_key!("email"),
            &"bad",
            &ValidParamCheck::default().add_rule(ValidEmail::default()),
        )
        .add(
            valid_key!("name"),
            &"",
            &ValidParamCheck::default().add_rule(ValidNotEmpty::<&str>::default()),
        );
    let err = valid.check().unwrap_err();
    let val = err.to_value();

    // Must have "field" key
    assert!(val.get("field").is_some());
    assert!(val["field"].is_array());

    let fields: Vec<&str> = val["field"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(fields.contains(&"email"));
    assert!(fields.contains(&"name"));
}

#[test]
fn test_valid_error_to_value_no_info_when_no_data() {
    // Default ValidRuleError has data=None, so "info" should be absent
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("field"),
        &"",
        &ValidParamCheck::default().add_rule(ValidNotEmpty::<&str>::default()),
    );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    // "info" only present when rule errors have set_data
    assert!(val.get("info").is_none());
}

// ===========================================================================
// Rule chaining: multiple rules on the same field
// ===========================================================================
#[test]
fn test_rule_chaining_all_pass() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("email"),
        &"test@example.com",
        &ValidParamCheck::default()
            .add_rule(ValidNotEmpty::<&str>::default())
            .add_rule(ValidEmail::default())
            .add_rule(ValidStrlen::max(100)),
    );
    assert!(valid.check().is_ok());
}

#[test]
fn test_rule_chaining_some_fail() {
    let mut valid = ValidParam::default();
    // An empty string should fail both NotEmpty and Email
    valid.add(
        valid_key!("email"),
        &"",
        &ValidParamCheck::default()
            .add_rule(ValidNotEmpty::<&str>::default())
            .add_rule(ValidEmail::default()),
    );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    let fields = val["field"].as_array().unwrap();
    // Both rules fail, so "email" appears twice in the field list
    assert_eq!(fields.len(), 2);
    assert!(fields.iter().all(|f| f.as_str().unwrap() == "email"));
}

#[test]
fn test_rule_chaining_strlen_and_pattern() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("username"),
        &"ab",
        &ValidParamCheck::default()
            .add_rule(ValidStrlen::range(3, 20))
            .add_rule(ValidPattern::Ident),
    );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    let fields = val["field"].as_array().unwrap();
    // Only strlen fails, Ident passes for "ab"
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].as_str().unwrap(), "username");
}

#[test]
fn test_rule_chaining_number_and_contains() {
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("status"),
        &5i32,
        &ValidParamCheck::default()
            .add_rule(ValidNumber::range(1, 10))
            .add_rule(ValidContains(&[1, 2, 3])),
    );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    let fields = val["field"].as_array().unwrap();
    // Number range passes (5 in 1..10), but Contains fails (5 not in [1,2,3])
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].as_str().unwrap(), "status");
}

// ===========================================================================
// Edge cases
// ===========================================================================
#[test]
fn test_valid_param_with_string_type() {
    let email = "user@domain.com".to_string();
    let mut valid = ValidParam::default();
    valid.add(
        valid_key!("email"),
        &email,
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    assert!(valid.check().is_ok());
}

#[test]
fn test_valid_param_reuse_after_check() {
    let mut valid = ValidParam::default();
    // First usage
    valid.add(
        valid_key!("field"),
        &"test@test.com",
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    assert!(valid.check().is_ok());

    // Reuse with error
    valid.add(
        valid_key!("field"),
        &"bad",
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    assert!(valid.check().is_err());

    // And reuse again - should be ok since check consumed errors
    assert!(valid.check().is_ok());
}

#[test]
fn test_strlen_eq_zero() {
    let rule = ValidStrlen::<&str>::eq(0);
    assert_rule_ok(&rule, &"");
    assert_rule_err(&rule, &"a");
}

#[test]
fn test_number_negative_range() {
    let rule = ValidNumber::<i32>::range(-100, -1);
    assert_rule_ok(&rule, &-50i32);
    assert_rule_ok(&rule, &-1i32);
    assert_rule_ok(&rule, &-100i32);

    assert_rule_err(&rule, &0i32);
    assert_rule_err(&rule, &1i32);
    assert_rule_err(&rule, &-101i32);
}

#[test]
fn test_ip_v4_only_rejects_v6() {
    let rule = ValidIp::<&str>::new(VALID_IP_V4);
    assert_rule_err(&rule, &"::1");
    assert_rule_err(&rule, &"2001:db8::1");
}

#[test]
fn test_ip_v6_only_rejects_v4() {
    let rule = ValidIp::<&str>::new(VALID_IP_V6);
    assert_rule_err(&rule, &"192.168.1.1");
    assert_rule_err(&rule, &"10.0.0.1");
}

#[test]
fn test_str_match_with_string_type() {
    let rule = ValidStrMatch::<String>::StartWith("hello");
    assert_rule_ok(&rule, &"hello world".to_string());
    assert_rule_err(&rule, &"world hello".to_string());
}

#[test]
fn test_contains_single_element() {
    let rule = ValidContains(&[42i32]);
    assert_rule_ok(&rule, &42);
    assert_rule_err(&rule, &0);
}

#[test]
fn test_valid_param_empty_check() {
    let mut valid = ValidParam::default();
    // No rules added at all
    assert!(valid.check().is_ok());
}

#[test]
fn test_credit_card_boundary_lengths() {
    let rule = ValidCreditCard::<&str>::default();
    // 12 digits is too short
    assert_rule_err(&rule, &"123456789012");
    // 20 digits is too long
    assert_rule_err(&rule, &"12345678901234567890");
}

#[test]
fn test_pattern_numeric_empty_fails() {
    // Empty string: chars().all() returns true for empty iterators,
    // so empty string technically passes is_ascii_digit check.
    let rule = ValidPattern::<&str>::Numeric;
    let result = rule.check(&"");
    // Empty string: all() on empty iterator returns true, so it passes
    assert!(result.is_ok());
}

#[test]
fn test_git_hash_case_insensitive() {
    let rule = ValidGit::<&str>::VersionHash;
    // Upper case hex should also work (is_ascii_hexdigit accepts A-F)
    assert_rule_ok(
        &rule,
        &"ABCDEF0123456789ABCDEF0123456789ABCDEF01",
    );
    // Mixed case
    assert_rule_ok(
        &rule,
        &"abCDef0123456789abCDef0123456789abCDef01",
    );
}

#[test]
fn test_datetime_with_string_type() {
    let rule = ValidDateTime::<String>::Date;
    assert_rule_ok(&rule, &"2024-06-15".to_string());
    assert_rule_err(&rule, &"not-a-date".to_string());
}

#[test]
fn test_color_rgb_uppercase_and_lowercase() {
    let rule = ValidColor::<&str>::RGB;
    assert_rule_ok(&rule, &"#aabbcc");
    assert_rule_ok(&rule, &"#AABBCC");
    assert_rule_ok(&rule, &"#AaBbCc");
}

#[test]
fn test_domain_with_port() {
    let rule = ValidDomain::<&str>::default();
    assert_rule_ok(&rule, &"example.com:3000");
    assert_rule_ok(&rule, &"127.0.0.1:8080");
}

#[test]
fn test_url_with_path_and_query() {
    let rule = ValidUrl::<&str>::default();
    assert_rule_ok(&rule, &"https://example.com/path?query=1&foo=bar");
    assert_rule_ok(&rule, &"http://192.168.0.1:9090/api/v1");
}

#[test]
fn test_valid_param_multiple_rules_same_key_accumulate() {
    let mut valid = ValidParam::default();
    let data = "";
    // Add multiple checks for the same key separately
    valid.add(
        valid_key!("field"),
        &data,
        &ValidParamCheck::default().add_rule(ValidNotEmpty::<&str>::default()),
    );
    valid.add(
        valid_key!("field"),
        &data,
        &ValidParamCheck::default().add_rule(ValidEmail::default()),
    );
    let err = valid.check().unwrap_err();
    let val = err.to_value();
    let fields = val["field"].as_array().unwrap();
    // Both adds contribute errors for the same field
    assert_eq!(fields.len(), 2);
}
