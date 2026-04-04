use serde::de::{self, Deserializer, Visitor};
use std::fmt;

// ==================== u64 ====================

pub fn deserialize_u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct U64StringVisitor;

    impl<'de> Visitor<'de> for U64StringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 or a string containing a u64")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 {
                return Err(E::custom("expected non-negative integer"));
            }
            Ok(value as u64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<u64>()
                .map_err(|_| E::custom("invalid u64 string"))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(U64StringVisitor)
}

pub fn deserialize_bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoolStringVisitor;

    impl<'de> Visitor<'de> for BoolStringVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean or a string containing 1/0/true/false")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value != 0)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value != 0)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value.trim() {
                "1" => Ok(true),
                "0" => Ok(false),
                _ => Err(E::custom("invalid bool string")),
            }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(BoolStringVisitor)
}

pub fn deserialize_option_u64_from_string<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionU64StringVisitor;

    impl<'de> Visitor<'de> for OptionU64StringVisitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, a u64, or a string containing a u64")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            deserialize_u64_from_string(deserializer).map(Some)
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            if value < 0 {
                return Err(E::custom("expected non-negative integer"));
            }
            Ok(Some(value as u64))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value.is_empty() {
                return Ok(None);
            }
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|_| E::custom("invalid u64 string"))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(OptionU64StringVisitor)
}

// ==================== i8 ====================

pub fn deserialize_i8_from_string<'de, D>(deserializer: D) -> Result<i8, D::Error>
where
    D: Deserializer<'de>,
{
    struct I8StringVisitor;

    impl<'de> Visitor<'de> for I8StringVisitor {
        type Value = i8;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an i8 or a string containing an i8")
        }

        fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < i8::MIN as i64 || value > i8::MAX as i64 {
                return Err(E::custom("i8 out of range"));
            }
            Ok(value as i8)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value > i8::MAX as u64 {
                return Err(E::custom("i8 out of range"));
            }
            Ok(value as i8)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<i8>()
                .map_err(|_| E::custom("invalid i8 string"))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(I8StringVisitor)
}

#[allow(dead_code)]
pub fn deserialize_option_i8_from_string<'de, D>(deserializer: D) -> Result<Option<i8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionI8StringVisitor;

    impl<'de> Visitor<'de> for OptionI8StringVisitor {
        type Value = Option<i8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, an i8, or a string containing an i8")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            deserialize_i8_from_string(deserializer).map(Some)
        }

        fn visit_i8<E: de::Error>(self, value: i8) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            if value < i8::MIN as i64 || value > i8::MAX as i64 {
                return Err(E::custom("i8 out of range"));
            }
            Ok(Some(value as i8))
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            if value > i8::MAX as u64 {
                return Err(E::custom("i8 out of range"));
            }
            Ok(Some(value as i8))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value.is_empty() {
                return Ok(None);
            }
            value
                .parse::<i8>()
                .map(Some)
                .map_err(|_| E::custom("invalid i8 string"))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(OptionI8StringVisitor)
}

// ==================== i64 ====================
#[allow(dead_code)]
pub fn deserialize_i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct I64StringVisitor;

    impl<'de> Visitor<'de> for I64StringVisitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an i64 or a string containing an i64")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value > i64::MAX as u64 {
                return Err(E::custom("i64 out of range"));
            }
            Ok(value as i64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<i64>()
                .map_err(|_| E::custom("invalid i64 string"))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(I64StringVisitor)
}
#[allow(dead_code)]
pub fn deserialize_option_i64_from_string<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionI64StringVisitor;

    impl<'de> Visitor<'de> for OptionI64StringVisitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, an i64, or a string containing an i64")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            deserialize_i64_from_string(deserializer).map(Some)
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            if value > i64::MAX as u64 {
                return Err(E::custom("i64 out of range"));
            }
            Ok(Some(value as i64))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value.is_empty() {
                return Ok(None);
            }
            value
                .parse::<i64>()
                .map(Some)
                .map_err(|_| E::custom("invalid i64 string"))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(OptionI64StringVisitor)
}

// ==================== u32 ====================

#[allow(dead_code)]
pub fn deserialize_u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    struct U32StringVisitor;

    impl<'de> Visitor<'de> for U32StringVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u32 or a string containing a u32")
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value > u32::MAX as u64 {
                return Err(E::custom("u32 out of range"));
            }
            Ok(value as u32)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 || value > u32::MAX as i64 {
                return Err(E::custom("expected non-negative integer within u32 range"));
            }
            Ok(value as u32)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<u32>()
                .map_err(|_| E::custom("invalid u32 string"))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(U32StringVisitor)
}

#[allow(dead_code)]
pub fn deserialize_option_u32_from_string<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionU32StringVisitor;

    impl<'de> Visitor<'de> for OptionU32StringVisitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, a u32, or a string containing a u32")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            deserialize_u32_from_string(deserializer).map(Some)
        }

        fn visit_u32<E: de::Error>(self, value: u32) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            if value > u32::MAX as u64 {
                return Err(E::custom("u32 out of range"));
            }
            Ok(Some(value as u32))
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            if value < 0 || value > u32::MAX as i64 {
                return Err(E::custom("expected non-negative integer within u32 range"));
            }
            Ok(Some(value as u32))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value.is_empty() {
                return Ok(None);
            }
            value
                .parse::<u32>()
                .map(Some)
                .map_err(|_| E::custom("invalid u32 string"))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(OptionU32StringVisitor)
}
