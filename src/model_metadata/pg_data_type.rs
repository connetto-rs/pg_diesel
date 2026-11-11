//! Submodule providing the `PgDataType` type, the declared type of a column
//! read from `pg_attribute` and `pg_type`.

use core::num::NonZeroU32;
use std::borrow::Cow;

use sql_traits::traits::{DataTypeLike, IntervalFields, VectorKind, VectorType};

use crate::{PgDieselDatabase, database::CatalogCache, models::PgType};

/// The declared type of a column, with every fact the catalogs hold about it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgDataType {
    /// The name the type reads as, as `PostgreSQL` and the parser spell it.
    name: String,
    /// The innermost element of the array, when the type is one.
    element: Option<Box<PgDataType>>,
    /// The final non-domain base of the type, when the type is a domain.
    base_type: Option<Box<PgDataType>>,
    /// The array dimensions the column declares.
    dimensions: Option<NonZeroU32>,
    /// The most characters a value of the type can hold.
    max_character_length: Option<u64>,
    /// The declared decimal digits, including fractional digits.
    numeric_precision: Option<u64>,
    /// The scale an exact decimal type pins.
    numeric_scale: Option<i64>,
    /// The digits of fractional seconds a time type keeps.
    fractional_seconds_precision: Option<u64>,
    /// The fields an interval restricts to.
    interval_fields: Option<IntervalFields>,
    /// The pgvector type and its dimension, when the type is one.
    vector: Option<VectorType>,
    /// Whether the type has a binary send function of its own, as
    /// `pg_type.typsend` records it, the base type's answer for a domain.
    send_function: Option<bool>,
    /// The subtype of the range type, or of the ranges a multirange type
    /// holds.
    range_subtype: Option<Box<PgDataType>>,
    /// The attributes of the composite type, in order.
    composite_fields: Option<Vec<PgDataType>>,
}

impl PgDataType {
    /// Builds the declared type a column records under `pg_type`.
    ///
    /// `typmod` is the `atttypmod` the column carries, `-1` when the type
    /// stores none. On an array it is the `atttypmod` of the element the
    /// array holds. `ndims` is the `attndims` the column carries, the number
    /// of array dimensions it declares.
    ///
    /// Returns `None` when an array element or a domain base is missing from
    /// `cache`.
    #[must_use]
    pub fn from_type(
        pg_type: &PgType,
        typmod: i32,
        cache: &CatalogCache,
        ndims: i32,
    ) -> Option<PgDataType> {
        Self::from_type_through(pg_type, typmod, cache, ndims, &[])
    }

    /// Builds the type under `pg_type`, where `in_progress` holds the OIDs of
    /// the composite types whose attributes are being read, so a composite
    /// that reaches itself through an array or a domain reads its own
    /// attributes once and stops.
    fn from_type_through(
        pg_type: &PgType,
        typmod: i32,
        cache: &CatalogCache,
        ndims: i32,
        in_progress: &[u32],
    ) -> Option<PgDataType> {
        if pg_type.typtype == "d" {
            let (base, effective_typmod, base_ndims) = final_base(pg_type, cache)?;
            let base_type =
                Self::from_type_through(base, effective_typmod, cache, base_ndims, in_progress)?;
            let send_function = base_type.send_function;
            return Some(PgDataType {
                name: cache
                    .type_name(pg_type.oid)
                    .map_or_else(|| pg_type.typname.clone(), ToOwned::to_owned),
                element: None,
                base_type: Some(Box::new(base_type)),
                dimensions: None,
                max_character_length: None,
                numeric_precision: None,
                numeric_scale: None,
                fractional_seconds_precision: None,
                interval_fields: None,
                vector: None,
                // `PostgreSQL` copies the send function into the domain, so
                // the domain answers its base type's.
                send_function,
                range_subtype: None,
                composite_fields: None,
            });
        }
        if pg_type.typcategory == "A" {
            let element_type = cache.pg_type(pg_type.typelem)?;
            let element = Self::from_type_through(element_type, typmod, cache, 0, in_progress)?;
            let dimensions = declared_dimensions(ndims);
            let name = cache.type_name(pg_type.oid).map_or_else(
                || array_name(&element, dimensions.map_or(1, NonZeroU32::get)),
                ToOwned::to_owned,
            );
            return Some(PgDataType {
                name,
                element: Some(Box::new(element)),
                dimensions,
                base_type: None,
                max_character_length: None,
                numeric_precision: None,
                numeric_scale: None,
                fractional_seconds_precision: None,
                interval_fields: None,
                vector: None,
                send_function: Some(pg_type.typsend != 0),
                range_subtype: None,
                composite_fields: None,
            });
        }
        Some(scalar(pg_type, typmod, cache, in_progress))
    }

    /// Whether `typname` names one of the integer types a serial column
    /// stores in, the ones the serial pseudo-types stand for.
    #[must_use]
    pub fn is_serial_integer(typname: &str) -> bool {
        matches!(typname, "int2" | "int4" | "int8")
    }
}

/// The size in bytes a character type's length is stored with.
const VARHDRSZ: i32 = 4;

/// The array dimensions a column declares, or `None` when it declares none.
fn declared_dimensions(ndims: i32) -> Option<NonZeroU32> {
    u32::try_from(ndims).ok().and_then(NonZeroU32::new)
}

/// The name of an array of `dimensions` elements, each of `element`.
fn array_name(element: &PgDataType, dimensions: u32) -> String {
    let mut name = element.name.clone();
    for _ in 0..dimensions {
        name.push_str("[]");
    }
    name
}

/// The final non-domain base of `domain`, with the type modifier and the
/// array dimension count the innermost domain over it pins.
///
/// `PostgreSQL` refuses a domain declared over itself, so the walk reaches a
/// base type, and the `typtypmod` and `typndims` of the domain directly over
/// the base are the ones the base sees.
fn final_base<'a>(
    mut domain: &'a PgType,
    cache: &'a CatalogCache,
) -> Option<(&'a PgType, i32, i32)> {
    let mut effective_typmod = -1;
    let mut ndims = 0;
    loop {
        if domain.typtype != "d" {
            return Some((domain, effective_typmod, ndims));
        }
        let base = cache.pg_type(domain.typbasetype)?;
        effective_typmod = domain.typtypmod;
        ndims = domain.typndims;
        domain = base;
    }
}

/// The facts of a scalar type, with its type modifier decoded.
fn scalar(pg_type: &PgType, typmod: i32, cache: &CatalogCache, in_progress: &[u32]) -> PgDataType {
    let name = cache
        .type_name(pg_type.oid)
        .unwrap_or(&pg_type.typname)
        .to_string();
    let (
        max_character_length,
        numeric_precision,
        numeric_scale,
        fractional_seconds_precision,
        interval_fields,
    ) = match pg_type.typname.as_str() {
        "varchar" | "bpchar" => (character_length(typmod), None, None, None, None),
        "numeric" => (
            None,
            numeric_precision(typmod),
            numeric_scale(typmod),
            None,
            None,
        ),
        "time" | "timetz" | "timestamp" | "timestamptz" => {
            (None, None, None, time_precision(typmod), None)
        }
        "interval" => {
            let (interval_fields, fractional_seconds_precision) = interval_parts(typmod);
            (
                None,
                None,
                None,
                fractional_seconds_precision,
                interval_fields,
            )
        }
        _ => (None, None, None, None, None),
    };
    PgDataType {
        send_function: Some(pg_type.typsend != 0),
        range_subtype: range_subtype(pg_type, cache, in_progress),
        composite_fields: composite_fields(pg_type, cache, in_progress),
        name,
        element: None,
        base_type: None,
        dimensions: None,
        max_character_length,
        numeric_precision,
        numeric_scale,
        fractional_seconds_precision,
        interval_fields,
        vector: vector_type(&pg_type.typname, typmod),
    }
}

/// The subtype a range or multirange type holds, read from `pg_range`, or
/// `None` for a type that is neither.
fn range_subtype(
    pg_type: &PgType,
    cache: &CatalogCache,
    in_progress: &[u32],
) -> Option<Box<PgDataType>> {
    if !(pg_type.typtype == "r" || pg_type.typtype == "m") {
        return None;
    }
    let subtype = cache.range_subtype(pg_type.oid)?;
    let subtype_type = cache.pg_type(subtype)?;
    Some(Box::new(PgDataType::from_type_through(
        subtype_type,
        -1,
        cache,
        0,
        in_progress,
    )?))
}

/// The attributes of a composite type, in order, read from the live
/// attributes of the relation `pg_type` records under `typrelid`, or `None`
/// for a type that is no composite, one whose relation the cache does not
/// hold, and one whose own attributes are being read, which a composite
/// reaching itself through an array or a domain reads once and stops.
fn composite_fields(
    pg_type: &PgType,
    cache: &CatalogCache,
    in_progress: &[u32],
) -> Option<Vec<PgDataType>> {
    if !(pg_type.typtype == "c" && pg_type.typrelid != 0) {
        return None;
    }
    if in_progress.contains(&pg_type.oid) {
        return None;
    }
    let attributes = cache.composite_attributes(pg_type.typrelid)?;
    let mut with_self = in_progress.to_vec();
    with_self.push(pg_type.oid);
    attributes
        .iter()
        .map(|(type_oid, field_typmod, field_ndims)| {
            let field_type = cache.pg_type(*type_oid)?;
            PgDataType::from_type_through(
                field_type,
                *field_typmod,
                cache,
                *field_ndims,
                &with_self,
            )
        })
        .collect()
}

/// The most characters a value of a length-modified type can hold, or `None`
/// when the type stores no length.
fn character_length(typmod: i32) -> Option<u64> {
    typmod
        .checked_sub(VARHDRSZ)
        .and_then(|length| u64::try_from(length).ok())
}

/// The decimal precision stored in the upper bits of a numeric type modifier.
fn numeric_precision(typmod: i32) -> Option<u64> {
    let value = u32::try_from(typmod.checked_sub(VARHDRSZ)?).ok()?;
    Some(u64::from((value >> 16) & 0xffff))
}

/// The scale an exact decimal type's modifier pins, or `None` when the
/// modifier stores none.
fn numeric_scale(typmod: i32) -> Option<i64> {
    let value = typmod.checked_sub(VARHDRSZ)?;
    if value < 0 {
        return None;
    }
    Some(i64::from(((value & 0x7ff) ^ 1024) - 1024))
}

/// The digits of fractional seconds a time type's modifier pins, or the
/// `PostgreSQL` default when the modifier stores none.
fn time_precision(typmod: i32) -> Option<u64> {
    let precision = if typmod < 0 { 6 } else { typmod.min(6) };
    u64::try_from(precision).ok()
}

/// The range bits `PostgreSQL` writes for an interval with no field
/// restriction, `INTERVAL_FULL_RANGE`, which a bare `interval(p)` carries.
const INTERVAL_FULL_RANGE: i32 = 0x7fff;

/// The fields an interval's modifier restricts to, with the digits of
/// fractional seconds it pins for a range that reaches `second`.
fn interval_parts(typmod: i32) -> (Option<IntervalFields>, Option<u64>) {
    if typmod < 0 {
        return (None, Some(6));
    }
    let range = (typmod >> 16) & 0x7fff;
    let precision = typmod & 0xffff;
    let fields = interval_fields(range);
    let reaches_seconds =
        fields.map_or(range == INTERVAL_FULL_RANGE, IntervalFields::holds_seconds);
    let fractional = reaches_seconds
        .then(|| {
            if precision == 0xffff {
                6
            } else {
                precision.min(6)
            }
        })
        .and_then(|precision| u64::try_from(precision).ok());
    (fields, fractional)
}

/// The fields a range the `PostgreSQL` interval modifier carries names.
///
/// Each field sets one bit at the index `PostgreSQL` assigns it, so a range
/// is the sum of the bits of the fields it holds.
fn interval_fields(range: i32) -> Option<IntervalFields> {
    match range {
        2 => Some(IntervalFields::Month),
        4 => Some(IntervalFields::Year),
        8 => Some(IntervalFields::Day),
        6 => Some(IntervalFields::YearToMonth),
        1032 => Some(IntervalFields::DayToHour),
        3080 => Some(IntervalFields::DayToMinute),
        7176 => Some(IntervalFields::DayToSecond),
        1024 => Some(IntervalFields::Hour),
        3072 => Some(IntervalFields::HourToMinute),
        7168 => Some(IntervalFields::HourToSecond),
        2048 => Some(IntervalFields::Minute),
        6144 => Some(IntervalFields::MinuteToSecond),
        4096 => Some(IntervalFields::Second),
        _ => None,
    }
}

/// The pgvector kind a type name spells.
fn vector_kind(typname: &str) -> Option<VectorKind> {
    match typname {
        "vector" => Some(VectorKind::Vector),
        "halfvec" => Some(VectorKind::HalfVec),
        "sparsevec" => Some(VectorKind::SparseVec),
        _ => None,
    }
}

/// The pgvector type and dimension a name and modifier hold, or `None` when
/// the name is not a pgvector type.
fn vector_type(typname: &str, typmod: i32) -> Option<VectorType> {
    let kind = vector_kind(typname)?;
    let dimensions = (typmod >= 0)
        .then(|| u32::try_from(typmod))
        .and_then(Result::ok);
    Some(VectorType::new(kind, dimensions))
}

impl DataTypeLike for PgDataType {
    type DB = PgDieselDatabase;

    fn name<'db>(&'db self, _database: &'db Self::DB) -> Cow<'db, str> {
        Cow::Borrowed(self.name.as_str())
    }

    fn max_character_length(&self, _database: &Self::DB) -> Option<u64> {
        self.max_character_length
    }

    fn numeric_precision(&self, _database: &Self::DB) -> Option<u64> {
        self.numeric_precision
    }

    fn numeric_scale(&self, _database: &Self::DB) -> Option<i64> {
        self.numeric_scale
    }

    fn fractional_seconds_precision(&self, _database: &Self::DB) -> Option<u64> {
        self.fractional_seconds_precision
    }

    fn interval_fields(&self, _database: &Self::DB) -> Option<IntervalFields> {
        self.interval_fields
    }

    fn vector(&self, _database: &Self::DB) -> Option<VectorType> {
        self.vector
    }

    fn array_element<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db Self> {
        self.element.as_deref()
    }

    fn array_dimensions(&self, _database: &Self::DB) -> Option<NonZeroU32> {
        self.dimensions
    }

    fn base_type<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db Self> {
        self.base_type.as_deref()
    }

    fn send_function(&self, _database: &Self::DB) -> Option<bool> {
        self.send_function
    }

    fn range_subtype<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db Self> {
        self.range_subtype.as_deref()
    }

    fn composite_fields<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> Option<impl Iterator<Item = &'db Self>> {
        self.composite_fields.as_deref().map(<[_]>::iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_length_decodes_the_written_length() {
        assert_eq!(character_length(-1), None);
        assert_eq!(character_length(3), None);
        assert_eq!(character_length(4), Some(0));
        assert_eq!(character_length(5), Some(1));
        assert_eq!(character_length(54), Some(50));
        assert_eq!(character_length(104), Some(100));
    }

    #[test]
    fn numeric_precision_decodes_digits_independently_of_scale() {
        for (precision, expected) in [(1, 1), (5, 5), (10, 10), (38, 38), (1000, 1000)] {
            for scale in [-1000, -2, 0, 2, 1000] {
                let typmod = ((precision << 16) | (scale & 0x7ff)) + VARHDRSZ;
                assert_eq!(numeric_precision(typmod), Some(expected));
                assert_eq!(numeric_scale(typmod), Some(i64::from(scale)));
            }
        }
    }

    #[test]
    fn numeric_precision_requires_a_modifier() {
        for typmod in [i32::MIN, -1, 0, VARHDRSZ - 1] {
            assert_eq!(numeric_precision(typmod), None);
        }
        assert_eq!(numeric_precision(VARHDRSZ), Some(0));
    }

    #[test]
    fn numeric_scale_decodes_precision_and_scale() {
        assert_eq!(numeric_scale(-1), None);
        assert_eq!(numeric_scale(3), None);
        assert_eq!(numeric_scale(4), Some(0));
        assert_eq!(numeric_scale(655_366), Some(2));
        assert_eq!(numeric_scale(327_684), Some(0));
        assert_eq!(numeric_scale(657_410), Some(-2));
    }

    #[test]
    fn time_precision_decodes_the_written_precision() {
        assert_eq!(time_precision(-1), Some(6));
        assert_eq!(time_precision(0), Some(0));
        assert_eq!(time_precision(3), Some(3));
        assert_eq!(time_precision(6), Some(6));
        assert_eq!(time_precision(9), Some(6));
    }

    #[test]
    fn interval_parts_decode_ranges_and_precisions() {
        assert_eq!(interval_parts(-1), (None, Some(6)));
        assert_eq!(
            interval_parts(458_751),
            (Some(IntervalFields::YearToMonth), None)
        );
        assert_eq!(
            interval_parts(470_351_871),
            (Some(IntervalFields::DayToSecond), Some(6))
        );
        assert_eq!(
            interval_parts(470_286_339),
            (Some(IntervalFields::DayToSecond), Some(3))
        );
        // `interval(3)` restricts no field and keeps three digits.
        assert_eq!(interval_parts(2_147_418_115), (None, Some(3)));
    }

    #[test]
    fn interval_fields_decode_every_range() {
        assert_eq!(interval_fields(2), Some(IntervalFields::Month));
        assert_eq!(interval_fields(4), Some(IntervalFields::Year));
        assert_eq!(interval_fields(8), Some(IntervalFields::Day));
        assert_eq!(interval_fields(6), Some(IntervalFields::YearToMonth));
        assert_eq!(interval_fields(1032), Some(IntervalFields::DayToHour));
        assert_eq!(interval_fields(3080), Some(IntervalFields::DayToMinute));
        assert_eq!(interval_fields(7176), Some(IntervalFields::DayToSecond));
        assert_eq!(interval_fields(1024), Some(IntervalFields::Hour));
        assert_eq!(interval_fields(3072), Some(IntervalFields::HourToMinute));
        assert_eq!(interval_fields(7168), Some(IntervalFields::HourToSecond));
        assert_eq!(interval_fields(2048), Some(IntervalFields::Minute));
        assert_eq!(interval_fields(6144), Some(IntervalFields::MinuteToSecond));
        assert_eq!(interval_fields(4096), Some(IntervalFields::Second));
        assert_eq!(interval_fields(0), None);
        assert_eq!(interval_fields(1), None);
    }

    #[test]
    fn serial_integer_covers_the_stored_types() {
        assert!(PgDataType::is_serial_integer("int2"));
        assert!(PgDataType::is_serial_integer("int4"));
        assert!(PgDataType::is_serial_integer("int8"));
        assert!(!PgDataType::is_serial_integer("varchar"));
    }

    #[test]
    fn vector_types_decode_the_dimension() {
        assert_eq!(
            vector_type("vector", 16),
            Some(VectorType::new(VectorKind::Vector, Some(16)))
        );
        assert_eq!(
            vector_type("vector", -1),
            Some(VectorType::new(VectorKind::Vector, None))
        );
        assert_eq!(
            vector_type("halfvec", 8),
            Some(VectorType::new(VectorKind::HalfVec, Some(8)))
        );
        assert_eq!(
            vector_type("sparsevec", 4),
            Some(VectorType::new(VectorKind::SparseVec, Some(4)))
        );
        assert_eq!(vector_type("int4", -1), None);
    }

    #[test]
    fn declared_dimensions_requires_a_positive_count() {
        assert_eq!(declared_dimensions(0), None);
        assert_eq!(declared_dimensions(-1), None);
        assert_eq!(declared_dimensions(1), NonZeroU32::new(1));
        assert_eq!(declared_dimensions(2), NonZeroU32::new(2));
    }

    #[test]
    fn an_array_name_appends_a_bracket_pair_per_dimension() {
        let element = PgDataType {
            name: "integer".to_string(),
            element: None,
            base_type: None,
            dimensions: None,
            max_character_length: None,
            numeric_precision: None,
            numeric_scale: None,
            fractional_seconds_precision: None,
            interval_fields: None,
            vector: None,
            send_function: None,
            range_subtype: None,
            composite_fields: None,
        };
        assert_eq!(array_name(&element, 1), "integer[]");
        assert_eq!(array_name(&element, 3), "integer[][][]");
    }
}
