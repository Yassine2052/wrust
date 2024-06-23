use crate::request::{RequestQueriesHashMap};

pub trait InjectStructTrait: 'static {
    fn init() -> Self
    where
        Self: Sized;

    fn from_hashmap(hashmap: &RequestQueriesHashMap) -> Self
        where
            Self: Sized;
}