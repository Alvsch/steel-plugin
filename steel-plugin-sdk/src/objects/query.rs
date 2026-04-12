use serde::de::DeserializeOwned;

use crate::objects::Entity;

/// A single field that can be fetched from an entity of type `E`.
pub trait QueryItem<E: Entity> {
    /// The Rust type returned when this field is fetched.
    type Output: DeserializeOwned;

    /// Converts this marker type into the wire enum variant for serialization.
    fn to_wire() -> E::WireQuery;
}

/// A set of fields to fetch in a single round-trip.
///
/// Implemented for tuples of [`QueryItem`]s via the [`impl_query_set!`] macro.
pub trait QuerySet<E: Entity> {
    /// The tuple of output types corresponding to each field in the set.
    type Output;

    /// Serializes all queries into a list of wire variants.
    fn to_wire() -> Vec<E::WireQuery>;

    /// Deserializes the positional msgpack array response into the output tuple.
    ///
    /// # Panics
    /// Panics if the response is malformed or the host returned unexpected types.
    fn from_response(data: &[u8]) -> Self::Output;
}

// Implement QuerySet for tuples of QueryItems
macro_rules! impl_query_set {
    ($($T:ident),+) => {
        impl<E, $($T),+> QuerySet<E> for ($($T,)+)
        where
            E: Entity,
            $($T: QueryItem<E>,)+
            ($($T::Output,)+): DeserializeOwned,
        {
            type Output = ($($T::Output,)+);

            fn to_wire() -> Vec<E::WireQuery> {
                vec![$($T::to_wire()),+]
            }

            fn from_response(data: &[u8]) -> Self::Output {
                rmp_serde::from_slice::<Self::Output>(data)
                    .expect("host returned malformed fetch response")
            }
        }
    };
}

impl_query_set!(Q0);
impl_query_set!(Q0, Q1);
impl_query_set!(Q0, Q1, Q2);
impl_query_set!(Q0, Q1, Q2, Q3);
impl_query_set!(Q0, Q1, Q2, Q3, Q4);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5, Q6);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10);
impl_query_set!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11);

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::{QueryItem, QuerySet};
    use crate::objects::Entity;

    struct TestEntity;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    enum TestQuery {
        Id,
        Name,
    }

    impl Entity for TestEntity {
        type WireQuery = TestQuery;
        type WireCommand = ();
    }

    struct Id;
    struct Name;

    impl QueryItem<TestEntity> for Id {
        type Output = u32;

        fn to_wire() -> TestQuery {
            TestQuery::Id
        }
    }

    impl QueryItem<TestEntity> for Name {
        type Output = String;

        fn to_wire() -> TestQuery {
            TestQuery::Name
        }
    }

    #[test]
    fn tuple_query_set_serializes_expected_wire_queries() {
        let wire = <(Id, Name) as QuerySet<TestEntity>>::to_wire();
        assert_eq!(wire, vec![TestQuery::Id, TestQuery::Name]);
    }

    #[test]
    fn tuple_query_set_deserializes_response_payload() {
        let payload = rmp_serde::to_vec(&(42_u32, String::from("steel")))
            .expect("response payload should serialize");

        let (id, name): (u32, String) =
            <(Id, Name) as QuerySet<TestEntity>>::from_response(&payload);
        assert_eq!(id, 42);
        assert_eq!(name, "steel");
    }

    #[test]
    #[should_panic(expected = "host returned malformed fetch response")]
    fn malformed_response_panics() {
        let _decoded: (u32, String) = <(Id, Name) as QuerySet<TestEntity>>::from_response(&[0xC0]);
    }
}
