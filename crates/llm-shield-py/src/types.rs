//! Type conversions between Rust and Python.
//!
//! This module provides utilities for converting between Rust types
//! and Python objects, ensuring seamless interoperability.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use llm_shield_core::{ScanResult, Entity, RiskFactor, Severity};
use serde_json::Value;

/// Convert a Rust ScanResult to a Python dictionary
///
/// This function converts the Rust ScanResult type to a Python dict
/// that can be easily used in Python code.
///
/// # Returns
///
/// A Python dict with keys:
/// - `sanitized_input`: str
/// - `is_valid`: bool
/// - `risk_score`: float
/// - `entities`: list[dict]
/// - `risk_factors`: list[dict]
pub fn scan_result_to_py(py: Python<'_>, result: &ScanResult) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);

    // Add basic fields
    dict.set_item("sanitized_input", &result.sanitized_input)?;
    dict.set_item("is_valid", result.is_valid)?;
    dict.set_item("risk_score", result.risk_score)?;

    // Convert entities to Python list
    let entities = PyList::empty_bound(py);
    for entity in &result.entities {
        let entity_dict = entity_to_py(py, entity)?;
        entities.append(entity_dict)?;
    }
    dict.set_item("entities", entities)?;

    // Convert risk factors to Python list
    let risk_factors = PyList::empty_bound(py);
    for factor in &result.risk_factors {
        let factor_dict = risk_factor_to_py(py, factor)?;
        risk_factors.append(factor_dict)?;
    }
    dict.set_item("risk_factors", risk_factors)?;

    Ok(dict.into())
}

/// Convert a Rust Entity to a Python dictionary
pub fn entity_to_py(py: Python<'_>, entity: &Entity) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);

    dict.set_item("entity_type", &entity.entity_type)?;
    dict.set_item("text", &entity.text)?;
    dict.set_item("start", entity.start)?;
    dict.set_item("end", entity.end)?;
    dict.set_item("score", entity.score)?;

    // Convert metadata
    let metadata = PyDict::new_bound(py);
    for (key, value) in &entity.metadata {
        metadata.set_item(key, value)?;
    }
    dict.set_item("metadata", metadata)?;

    Ok(dict.into())
}

/// Convert a Rust RiskFactor to a Python dictionary
pub fn risk_factor_to_py(py: Python<'_>, factor: &RiskFactor) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);

    dict.set_item("factor_type", &factor.factor_type)?;
    dict.set_item("description", &factor.description)?;
    dict.set_item("severity", severity_to_str(&factor.severity))?;
    dict.set_item("score_contribution", factor.score_contribution)?;

    Ok(dict.into())
}

/// Convert Severity enum to string
fn severity_to_str(severity: &Severity) -> &'static str {
    match severity {
        Severity::None => "none",
        Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}

/// Convert a Python dict to JSON Value for config parsing
pub fn py_dict_to_json(dict: &Bound<'_, PyDict>) -> PyResult<Value> {
    let mut map = serde_json::Map::new();

    for (key, value) in dict.iter() {
        let key_str: String = key.extract()?;
        let json_value = py_any_to_json(&value)?;
        map.insert(key_str, json_value);
    }

    Ok(Value::Object(map))
}

/// Convert any Python object to JSON Value
fn py_any_to_json(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    // Try different type conversions
    if let Ok(val) = obj.extract::<bool>() {
        Ok(Value::Bool(val))
    } else if let Ok(val) = obj.extract::<i64>() {
        Ok(Value::Number(val.into()))
    } else if let Ok(val) = obj.extract::<f64>() {
        Ok(Value::Number(
            serde_json::Number::from_f64(val).unwrap_or_else(|| 0.into())
        ))
    } else if let Ok(val) = obj.extract::<String>() {
        Ok(Value::String(val))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        py_dict_to_json(dict)
    } else if let Ok(list) = obj.downcast::<PyList>() {
        py_list_to_json(list)
    } else if obj.is_none() {
        Ok(Value::Null)
    } else {
        Ok(Value::Null)
    }
}

/// Convert Python list to JSON array
fn py_list_to_json(list: &Bound<'_, PyList>) -> PyResult<Value> {
    let mut arr = Vec::new();

    for item in list.iter() {
        let json_value = py_any_to_json(&item)?;
        arr.push(json_value);
    }

    Ok(Value::Array(arr))
}

/// Parse scanner configuration from Python dict
pub fn parse_config<T>(dict: Option<&Bound<'_, PyDict>>) -> PyResult<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    match dict {
        Some(d) => {
            let json_value = py_dict_to_json(d)?;
            serde_json::from_value(json_value)
                .map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(
                        format!("Invalid configuration: {}", e)
                    )
                })
        }
        None => Ok(T::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_result_conversion() {
        Python::with_gil(|py| {
            let result = ScanResult {
                sanitized_input: "test".to_string(),
                is_valid: true,
                risk_score: 0.1,
                entities: vec![],
                risk_factors: vec![],
            };

            let py_dict = scan_result_to_py(py, &result).unwrap();
            let dict = py_dict.bind(py);

            assert_eq!(
                dict.get_item("is_valid").unwrap().unwrap().extract::<bool>().unwrap(),
                true
            );
            assert_eq!(
                dict.get_item("risk_score").unwrap().unwrap().extract::<f64>().unwrap(),
                0.1
            );
        });
    }

    #[test]
    fn test_entity_conversion() {
        Python::with_gil(|py| {
            let entity = Entity {
                entity_type: "TEST".to_string(),
                text: "example".to_string(),
                start: 0,
                end: 7,
                score: 0.95,
                metadata: std::collections::HashMap::new(),
            };

            let py_dict = entity_to_py(py, &entity).unwrap();
            let dict = py_dict.bind(py);

            assert_eq!(
                dict.get_item("entity_type").unwrap().unwrap().extract::<String>().unwrap(),
                "TEST"
            );
            assert_eq!(
                dict.get_item("score").unwrap().unwrap().extract::<f64>().unwrap(),
                0.95
            );
        });
    }
}
