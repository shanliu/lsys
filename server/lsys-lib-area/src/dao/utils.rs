const CLEAR_WORD: &[(&str, &str)] = &[
    ("-", "-"),
    ("社区居委会", "居委会"),
    ("居委会", "居委会"),
    ("片区街道", "片区街道"),
    ("街道办", "办"),
    ("村委会", "委会"),
    ("村村", "村"),
    ("民委员会", "民委员会"),
    ("省直辖县级行政区划", "级行政区划"),
    ("省直辖市级行政区划", "级行政区划"),
];
const BAD_WORD: &[&str] = &[
    "-",
    "开发区",
    "自治县",
    "自治州",
    "自治区",
    "特别行政区",
    "自治区",
    "直辖县级行政区划",
    "片区街道",
    "街道办",
];

pub(crate) fn key_word_clear(name: &str) -> &str {
    let name = name.trim();
    if name.chars().count() <= 2 {
        return name;
    }
    for tmp in BAD_WORD {
        if name.ends_with(tmp) {
            return name.trim_end_matches(tmp);
        }
    }
    for tmp in CLEAR_WORD {
        if name.ends_with(tmp.0) {
            return name.trim_end_matches(tmp.1);
        }
    }
    name
}

pub(crate) fn name_clear(name: &str) -> String {
    let mut tmp = name.trim();
    for tcc in CLEAR_WORD {
        if tmp.ends_with(tcc.0) {
            tmp = tmp.trim_end_matches(tcc.1);
        }
    }
    tmp.to_owned()
}
