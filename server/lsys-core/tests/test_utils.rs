use lsys_core::utils::*;

// ─── RandType tests ───────────────────────────────────────────────────────────

#[test]
fn rand_str_number_only_digits() {
    for len in [1, 4, 8, 16, 32] {
        let s = rand_str(RandType::Number, len);
        assert_eq!(s.len(), len);
        assert!(s.chars().all(|c| c.is_ascii_digit()), "got non-digit: {s}");
    }
}

#[test]
fn rand_str_upper_only_uppercase() {
    let s = rand_str(RandType::Upper, 20);
    assert_eq!(s.len(), 20);
    assert!(s.chars().all(|c| c.is_ascii_uppercase()));
}

#[test]
fn rand_str_lower_only_lowercase() {
    let s = rand_str(RandType::Lower, 20);
    assert_eq!(s.len(), 20);
    assert!(s.chars().all(|c| c.is_ascii_lowercase()));
}

#[test]
fn rand_str_upper_number_charset() {
    let s = rand_str(RandType::UpperNumber, 40);
    assert_eq!(s.len(), 40);
    assert!(
        s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    );
}

#[test]
fn rand_str_lower_number_charset() {
    let s = rand_str(RandType::LowerNumber, 40);
    assert_eq!(s.len(), 40);
    assert!(
        s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    );
}

#[test]
fn rand_str_upper_hex_charset() {
    let s = rand_str(RandType::UpperHex, 40);
    assert_eq!(s.len(), 40);
    assert!(
        s.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase())
    );
}

#[test]
fn rand_str_lower_hex_charset() {
    let s = rand_str(RandType::LowerHex, 40);
    assert_eq!(s.len(), 40);
    assert!(
        s.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
}

#[test]
fn rand_str_vaild_code_digits_and_length() {
    for len in [1, 4, 6, 8, 12] {
        let s = rand_str(RandType::VaildCode, len);
        assert_eq!(s.len(), len);
        assert!(s.chars().all(|c| c.is_ascii_digit()), "got non-digit: {s}");
    }
}

#[test]
fn rand_str_zero_length() {
    let s = rand_str(RandType::Number, 0);
    assert!(s.is_empty());
}

// ─── now_time tests ───────────────────────────────────────────────────────────

#[test]
fn now_time_returns_reasonable_timestamp() {
    let ts = now_time().expect("now_time should succeed");
    // Timestamp should be after 2020-01-01 and before 2100-01-01
    assert!(ts > 1_577_836_800, "timestamp too small: {ts}");
    assert!(ts < 4_102_444_800, "timestamp too large: {ts}");
}

#[test]
fn now_time_monotonic_within_call() {
    let t1 = now_time().unwrap();
    let t2 = now_time().unwrap();
    assert!(t2 >= t1);
}

// ─── str_time tests ───────────────────────────────────────────────────────────

#[test]
fn str_time_valid_datetime() {
    let dt = str_time("2024-01-15 14:30:00").expect("should parse");
    assert_eq!(
        dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        "2024-01-15 14:30:00"
    );
}

#[test]
fn str_time_midnight() {
    let dt = str_time("2000-01-01 00:00:00").expect("should parse midnight");
    assert_eq!(dt.format("%H:%M:%S").to_string(), "00:00:00");
}

#[test]
fn str_time_invalid_format() {
    assert!(str_time("not-a-date").is_err());
}

#[test]
fn str_time_empty_string() {
    assert!(str_time("").is_err());
}

#[test]
fn str_time_date_only_missing_time() {
    assert!(str_time("2024-01-15").is_err());
}

#[test]
fn str_time_invalid_month() {
    assert!(str_time("2024-13-01 00:00:00").is_err());
}

// ─── string_clear tests ──────────────────────────────────────────────────────

#[test]
fn string_clear_space_trims_and_squashes() {
    let result = string_clear(
        "  hello   world  ",
        StringClear::Option(STRING_CLEAR_SPACE),
        None,
    );
    assert_eq!(result, "hello world");
}

#[test]
fn string_clear_space_tab_becomes_single_space() {
    // Tabs are whitespace, split_whitespace handles them
    let result = string_clear("a\tb", StringClear::Option(STRING_CLEAR_SPACE), None);
    assert_eq!(result, "a b");
}

#[test]
fn string_clear_nl_removes_newlines() {
    let result = string_clear(
        "line1\nline2\rline3",
        StringClear::Option(STRING_CLEAR_NL),
        None,
    );
    assert_eq!(result, "line1 line2 line3");
}

#[test]
fn string_clear_format_removes_tabs_null_backslash_and_newlines() {
    let input = "a\tb\0c\\d\ne\rf";
    let result = string_clear(input, StringClear::Option(STRING_CLEAR_FORMAT), None);
    // FORMAT includes SPACE | NL, so tabs/null/backslash → space, newlines → space, then squashed
    assert_eq!(result, "a b c d e f");
}

#[test]
fn string_clear_xss_replaces_angle_brackets_and_ampersand() {
    let input = "<script>alert('xss');</script>&foo";
    let result = string_clear(input, StringClear::Option(STRING_CLEAR_XSS), None);
    assert!(!result.contains('<'));
    assert!(!result.contains('>'));
    assert!(!result.contains('&'));
    assert!(result.contains('['));
    assert!(result.contains(']'));
}

#[test]
fn string_clear_xss_specific_replacement() {
    assert_eq!(
        string_clear("<b>&</b>", StringClear::Option(STRING_CLEAR_XSS), None),
        "[b] [/b]"
    );
}

#[test]
fn string_clear_combined_xss_and_space() {
    let input = "  <b>  hello  </b>  ";
    let result = string_clear(
        input,
        StringClear::Option(STRING_CLEAR_XSS | STRING_CLEAR_SPACE),
        None,
    );
    assert!(!result.contains('<'));
    assert!(!result.contains('>'));
    // Spaces should be squashed and trimmed
    assert!(!result.starts_with(' '));
    assert!(!result.ends_with(' '));
    assert!(!result.contains("  "));
}

#[test]
fn string_clear_combined_format_and_xss() {
    let input = "<div>\thello\n</div>";
    let result = string_clear(
        input,
        StringClear::Option(STRING_CLEAR_FORMAT | STRING_CLEAR_XSS),
        None,
    );
    assert!(!result.contains('<'));
    assert!(!result.contains('>'));
    assert!(!result.contains('\t'));
    assert!(!result.contains('\n'));
}

#[test]
fn string_clear_like_keyword_escapes_percent_and_underscore() {
    // LikeKeyWord first escapes %, _ with backslash, then applies FORMAT which
    // replaces backslashes with spaces and squashes whitespace.
    let result = string_clear("100%_done", StringClear::LikeKeyWord, None);
    // Original '%' and '_' should still be present (no longer raw wildcards)
    assert!(result.contains('%'));
    assert!(result.contains('_'));
    // The result should not contain raw backslashes (FORMAT removes them)
    assert!(!result.contains('\\'));
}

#[test]
fn string_clear_like_keyword_replaces_backslash() {
    let result = string_clear("path\\to\\file", StringClear::LikeKeyWord, None);
    // Backslashes are replaced with spaces, then FORMAT is applied
    assert!(!result.contains('\\'));
}

#[test]
fn string_clear_ident_keeps_valid_chars() {
    let result = string_clear("hello-world_v1.0", StringClear::Ident, None);
    assert_eq!(result, "hello-world_v1.0");
}

#[test]
fn string_clear_ident_strips_invalid_chars() {
    let result = string_clear("he!l@l#o$w%o^r&l*d", StringClear::Ident, None);
    assert_eq!(result, "helloworld");
}

#[test]
fn string_clear_ident_strips_spaces_and_special() {
    let result = string_clear("  user name!  ", StringClear::Ident, None);
    assert_eq!(result, "username");
}

#[test]
fn string_clear_take_truncates_output() {
    let result = string_clear("abcdefghij", StringClear::Option(0), Some(5));
    assert_eq!(result, "abcde");
}

#[test]
fn string_clear_take_with_space_flag() {
    let result = string_clear(
        "  hello   world  ",
        StringClear::Option(STRING_CLEAR_SPACE),
        Some(7),
    );
    // After clearing: "hello world", then take 7 → "hello w"
    assert_eq!(result, "hello w");
}

#[test]
fn string_clear_take_zero() {
    let result = string_clear("hello", StringClear::Option(0), Some(0));
    assert!(result.is_empty());
}

#[test]
fn string_clear_take_exceeds_length() {
    let result = string_clear("hi", StringClear::Option(0), Some(100));
    assert_eq!(result, "hi");
}

#[test]
fn string_clear_empty_input() {
    assert_eq!(
        string_clear("", StringClear::Option(STRING_CLEAR_SPACE), None),
        ""
    );
    assert_eq!(string_clear("", StringClear::Ident, None), "");
    assert_eq!(string_clear("", StringClear::LikeKeyWord, None), "");
}

#[test]
fn string_clear_option_zero_flags_passthrough() {
    let result = string_clear("hello  world\n", StringClear::Option(0), None);
    assert_eq!(result, "hello  world\n");
}

#[test]
fn string_clear_ident_unicode_stripped() {
    let result = string_clear("café☕", StringClear::Ident, None);
    // is_alphanumeric includes unicode letters, so 'c','a','f','é' pass; '☕' is not alphanumeric
    assert!(result.contains("caf"));
    assert!(!result.contains('☕'));
}

// ─── VecStringJoin tests ─────────────────────────────────────────────────────

#[test]
fn vec_join_integers() {
    let v = vec![1, 2, 3];
    assert_eq!(v.string_join(","), "1,2,3");
}

#[test]
fn vec_join_strings() {
    let v = vec!["hello", "world"];
    assert_eq!(v.string_join(" "), "hello world");
}

#[test]
fn vec_join_string_owned() {
    let v = vec!["a".to_string(), "b".to_string()];
    assert_eq!(v.string_join("-"), "a-b");
}

#[test]
fn vec_join_single_element() {
    let v = vec![42];
    assert_eq!(v.string_join(","), "42");
}

#[test]
fn vec_join_empty() {
    let v: Vec<i32> = vec![];
    assert_eq!(v.string_join(","), "");
}

#[test]
fn vec_join_empty_separator() {
    let v = vec![1, 2, 3];
    assert_eq!(v.string_join(""), "123");
}

#[test]
fn vec_join_multi_char_separator() {
    let v = vec!["a", "b", "c"];
    assert_eq!(v.string_join(" | "), "a | b | c");
}

#[test]
fn vec_join_floats() {
    let v = vec![1.5, 2.5];
    let joined = v.string_join(",");
    assert!(joined.contains("1.5"));
    assert!(joined.contains("2.5"));
}

// ─── RequestEnv tests ────────────────────────────────────────────────────────

#[test]
fn request_env_all_none() {
    let env = RequestEnv::new(None, None, None, None, None).expect("all-None should succeed");
    assert!(env.request_lang.is_none());
    assert!(env.request_ip.is_none());
    assert!(env.request_id.is_none());
    assert!(env.request_user_agent.is_none());
    assert!(env.device_id.is_none());
    assert!(env.request_time > 0);
}

#[test]
fn request_env_valid_all_fields() {
    let env = RequestEnv::new(
        Some("en-US"),
        Some("192.168.1.1"),
        Some("req-12345678"),
        Some("Mozilla/5.0"),
        Some("device-1"),
    )
    .expect("valid params should succeed");

    assert_eq!(env.request_lang.as_deref(), Some("en-US"));
    assert_eq!(env.request_ip.as_deref(), Some("192.168.1.1"));
    assert_eq!(env.request_id.as_deref(), Some("req-12345678"));
    assert_eq!(env.request_user_agent.as_deref(), Some("Mozilla/5.0"));
    assert_eq!(env.device_id.as_deref(), Some("device-1"));
    assert!(env.request_time > 0);
}

#[test]
fn request_env_valid_ipv6() {
    let env = RequestEnv::new(None, Some("::1"), None, None, None);
    assert!(env.is_ok());
}

#[test]
fn request_env_valid_ipv4() {
    let env = RequestEnv::new(None, Some("10.0.0.1"), None, None, None);
    assert!(env.is_ok());
}

#[test]
fn request_env_invalid_ip() {
    let result = RequestEnv::new(None, Some("not-an-ip"), None, None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_invalid_ip_extra_octets() {
    let result = RequestEnv::new(None, Some("1.2.3.4.5"), None, None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_lang_too_short() {
    // Minimum length is 5
    let result = RequestEnv::new(Some("en"), None, None, None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_lang_too_long() {
    // Maximum length is 12
    let result = RequestEnv::new(Some("en-US-extra-long"), None, None, None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_lang_valid_boundary() {
    // Exactly 5 chars – minimum
    let env = RequestEnv::new(Some("zh-CN"), None, None, None, None);
    assert!(env.is_ok());
}

#[test]
fn request_env_lang_with_format_chars() {
    // NotFormat rejects tabs, newlines, etc.
    let result = RequestEnv::new(Some("en-US\t"), None, None, None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_request_id_too_short() {
    // Minimum length is 8
    let result = RequestEnv::new(None, None, Some("short"), None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_request_id_valid_ident() {
    let env = RequestEnv::new(None, None, Some("abcd-1234"), None, None);
    assert!(env.is_ok());
}

#[test]
fn request_env_request_id_invalid_chars() {
    // Ident pattern: alphanumeric, dash, dot, underscore only
    let result = RequestEnv::new(None, None, Some("req id!@#$%^&*"), None, None);
    assert!(result.is_err());
}

#[test]
fn request_env_user_agent_too_long() {
    // Maximum length is 254
    let long_ua = "A".repeat(255);
    let result = RequestEnv::new(None, None, None, Some(&long_ua), None);
    assert!(result.is_err());
}

#[test]
fn request_env_user_agent_valid() {
    let env = RequestEnv::new(
        None,
        None,
        None,
        Some("Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/120.0"),
        None,
    );
    assert!(env.is_ok());
}

#[test]
fn request_env_device_id_too_long() {
    // Maximum length is 64
    let long_id = "d".repeat(65);
    let result = RequestEnv::new(None, None, None, None, Some(&long_id));
    assert!(result.is_err());
}

#[test]
fn request_env_device_id_valid() {
    let env = RequestEnv::new(None, None, None, None, Some("dev-abc-123"));
    assert!(env.is_ok());
}

#[test]
fn request_env_device_id_with_format_chars() {
    let result = RequestEnv::new(None, None, None, None, Some("dev\nid"));
    assert!(result.is_err());
}

#[test]
fn request_env_sets_request_time() {
    let env = RequestEnv::new(None, None, None, None, None).unwrap();
    let now = now_time().unwrap();
    // request_time should be very close to now (within 2 seconds)
    assert!(env.request_time <= now);
    assert!(now - env.request_time < 2);
}
