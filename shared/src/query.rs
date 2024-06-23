use std::collections::HashMap;
use crate::query::QueryParamValue::{Multiple, Single};
use crate::query::QueryParamValueType::{Str, Int, Float, Boolean, UInt};

pub type QueriesHashMap = HashMap<String, QueryParamType>;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryParamValueType {
    Str(String),
    Int(isize),
    UInt(usize),
    Float(f64),
    Boolean(bool)
}

#[derive(Debug, Clone)]
pub enum QueryParamValue {
    Single(QueryParamValueType),
    Multiple(Vec<QueryParamValueType>)
}

#[derive(Debug)]
pub struct Flags {
    pub is_optional: bool,
    pub is_array: bool,
    pub allow_empty: bool
}

#[derive(Debug)]
pub struct QueryParamType {
    pub _type: QueryParamValueType,
    pub flags: Flags
}

#[derive(Debug, Clone)]
pub struct QueryParam {
    pub value: QueryParamValue
}

impl QueryParam {
    pub fn add_value(&mut self, string_value: String, _type: QueryParamValueType) -> Option<Self> {
        let data = Self::generate_data(&string_value, _type);

        if let Some(data) = data {
            let value =  match &self.value {
                Single(value) => {
                    Single(value.clone())
                },
                Multiple(value) => {
                    let mut array = value.clone();
                    array.push(data);
                    Multiple(array)
                }
            };

            return Some(
                Self {
                    value
                }
            );
        }

        None
    }

    pub fn from(string_value: String, _type: QueryParamValueType, is_array: bool) -> Option<Self> {
        let data = Self::generate_data(&string_value, _type);

        if let Some(data) = data {
            let value = if is_array {
                Multiple(Vec::from([data]))
            } else {
                Single(data)
            };

            return Some(
                Self {
                    value
                }
            );
        }

        None
    }

    fn generate_data(string_value: &String, _type: QueryParamValueType) -> Option<QueryParamValueType> {
        let mut data = None;

        match _type {
            Str(_) => {
                data = Some(Str(string_value.clone()));
            },
            Int(_) => {
                if let Ok(value) = string_value.parse::<isize>() {
                    data = Some(Int(value));
                }
            },
            UInt(_) => {
                if let Ok(value) = string_value.parse::<usize>() {
                    data = Some(UInt(value));
                }
            }
            Float(_) => {
                if let Ok(value) = string_value.parse::<f64>() {
                    data = Some(Float(value));
                }
            },
            Boolean(_) => {
                let value = matches!(string_value.as_str(), "true" | "t" | "1");
                data = Some(Boolean(value));
            }
        };

        data
    }
}