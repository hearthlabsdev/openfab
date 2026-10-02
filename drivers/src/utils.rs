use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::errors::ConfigError;
use crate::config::ConfigSchema;

pub fn config_to_json(
    schema: &ConfigSchema,
    form: &HashMap<String, String>,
) -> Result<Value, ConfigError> {
    let mut object = Map::new();

    for field in &schema.fields {
        let value = match form.get(&field.name) {
            Some(value) => value,
            None => {
                if field.required.unwrap_or(false) {
                    return Err(ConfigError::MissingField(field.name.clone()));
                }

                continue;
            }
        };

        let json_value = match field.field_type.as_str() {
            "text" | "password" => {
                Value::String(value.clone())
            }

            "number" => {
                Value::Number(
                    value
                        .parse::<u64>()
                        .map_err(|_| {
                            ConfigError::InvalidField(field.name.clone())
                        })?
                        .into()
                )
            }

            "boolean" => {
                Value::Bool(
                    value
                        .parse::<bool>()
                        .map_err(|_| {
                            ConfigError::InvalidField(field.name.clone())
                        })?
                )
            }

            other => {
                return Err(ConfigError::UnknownFieldType(
                    other.to_string()
                ));
            }
        };

        object.insert(field.name.clone(), json_value);
    }

    Ok(Value::Object(object))
}

pub fn get_value_type(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(_) => "bool".into(),
        Value::Number(_) => "number".into(),
        Value::String(_) => "string".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
    }
}