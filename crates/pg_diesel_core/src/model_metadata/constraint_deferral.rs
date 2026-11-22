//! Decoding of the deferral flags `PostgreSQL` records beside a constraint.

use sql_traits::structs::ConstraintDeferral;

/// Reads the deferral a constraint's `DEFERRABLE` and `INITIALLY DEFERRED`
/// flags record, `NOT DEFERRABLE` outweighing `INITIALLY DEFERRED` as in
/// `sql_traits`.
pub(crate) fn decode_deferral(deferrable: bool, initially_deferred: bool) -> ConstraintDeferral {
    match (deferrable, initially_deferred) {
        (false, _) => ConstraintDeferral::NotDeferrable,
        (true, false) => ConstraintDeferral::InitiallyImmediate,
        (true, true) => ConstraintDeferral::InitiallyDeferred,
    }
}
