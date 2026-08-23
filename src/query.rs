//! JQ-style query language for filtering and transforming JSON values.
//!
//! Uses jaq (a jq clone) to provide powerful JSON filtering capabilities.
//! Generic enough for any binary that needs to filter/query JSON records
//! (log entries, doc records, repo maps, etc.), not just viewer servers.
//!
//! # Example Queries
//!
//! ```text
//! # Filter by field value
//! select(.level == "ERROR")
//!
//! # Multiple conditions
//! select(.level == "ERROR" and .span_name == "my_function")
//!
//! # Search in text field
//! select(.message | contains("panic"))
//!
//! # Combine with regex
//! select(.message | test("error|panic"; "i"))
//!
//! # Filter by field existence
//! select(.fields.some_key != null)
//!
//! # Extract specific fields
//! {level, message, timestamp}
//!
//! # Filter by array element
//! select(.tags | any(. == "testing"))
//! ```

use jaq_interpret::{
    Ctx,
    FilterT,
    ParseCtx,
    RcIter,
    Val,
};
use serde_json::Value;

/// Error type for query operations
#[derive(Debug)]
pub struct QueryError {
    pub message: String,
}

impl std::fmt::Display for QueryError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for QueryError {}

/// A compiled jq filter ready to execute
pub struct JqFilter {
    filter: jaq_interpret::Filter,
}

impl JqFilter {
    /// Compile a jq filter string
    pub fn compile(query: &str) -> Result<Self, QueryError> {
        // Create parsing context and add standard definitions
        let mut ctx = ParseCtx::new(Vec::new());
        ctx.insert_natives(jaq_core::core());
        ctx.insert_defs(jaq_std::std());

        // Parse the query - jaq_syn::parse returns Option<T>
        let parsed = jaq_syn::parse(query, |p| p.module(|p| p.term()));

        let main = parsed.ok_or_else(|| QueryError {
            message: "Parse error: invalid jq expression".to_string(),
        })?;

        // Convert and compile
        let filter = main.conv(query);
        let filter = ctx.compile(filter);

        if !ctx.errs.is_empty() {
            let msg = ctx
                .errs
                .iter()
                .map(|(e, _span)| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(QueryError {
                message: format!("Compilation error: {}", msg),
            });
        }

        Ok(Self { filter })
    }

    /// Run the filter on a JSON value, returning all outputs
    pub fn run(
        &self,
        input: &Value,
    ) -> Vec<Result<Value, String>> {
        let inputs = RcIter::new(std::iter::empty());
        let val = Val::from(input.clone());
        let ctx = Ctx::new([], &inputs);

        self.filter
            .run((ctx, val))
            .map(|result| {
                result
                    .map(|v| serde_json::Value::from(v))
                    .map_err(|e| format!("{}", e))
            })
            .collect()
    }

    /// Run the filter on a value, returning true if any output is truthy
    pub fn matches(
        &self,
        input: &Value,
    ) -> bool {
        let results = self.run(input);
        results.into_iter().any(|r| match r {
            Ok(Value::Bool(true)) => true,
            Ok(Value::Null) | Ok(Value::Bool(false)) => false,
            Ok(_) => true, // Non-null, non-false values are truthy
            Err(_) => false,
        })
    }
}

/// Filter a slice of JSON values using a jq query (returns matching values)
pub fn filter_values<'a>(
    values: impl IntoIterator<Item = &'a Value>,
    query: &str,
) -> Result<Vec<Value>, QueryError> {
    let filter = JqFilter::compile(query)?;

    let mut results = Vec::new();
    for value in values {
        if filter.matches(value) {
            results.push(value.clone());
        }
    }

    Ok(results)
}

/// Transform values using a jq query (can produce multiple/different outputs per input)
pub fn transform_values<'a>(
    values: impl IntoIterator<Item = &'a Value>,
    query: &str,
) -> Result<Vec<Value>, QueryError> {
    let filter = JqFilter::compile(query)?;

    let mut results = Vec::new();
    for value in values {
        for result in filter.run(value) {
            if let Ok(v) = result {
                results.push(v);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compile_simple_filter() {
        let filter = JqFilter::compile(".");
        assert!(filter.is_ok());
    }

    #[test]
    fn test_compile_invalid_filter() {
        let filter = JqFilter::compile("[invalid");
        assert!(filter.is_err());
    }

    #[test]
    fn test_filter_select() {
        let filter = JqFilter::compile("select(.level == \"ERROR\")").unwrap();

        let error_entry = json!({"level": "ERROR", "message": "fail"});
        let info_entry = json!({"level": "INFO", "message": "ok"});

        assert!(filter.matches(&error_entry));
        assert!(!filter.matches(&info_entry));
    }

    #[test]
    fn test_filter_contains() {
        let filter =
            JqFilter::compile("select(.message | contains(\"panic\"))")
                .unwrap();

        let match_entry = json!({"message": "thread panic detected"});
        let no_match = json!({"message": "all good"});

        assert!(filter.matches(&match_entry));
        assert!(!filter.matches(&no_match));
    }

    #[test]
    fn test_filter_case_insensitive() {
        let filter =
            JqFilter::compile("select(.message | test(\"error\"; \"i\"))")
                .unwrap();

        let upper = json!({"message": "Error occurred"});
        let lower = json!({"message": "error occurred"});
        let no_match = json!({"message": "all good"});

        assert!(filter.matches(&upper));
        assert!(filter.matches(&lower));
        assert!(!filter.matches(&no_match));
    }

    #[test]
    fn test_filter_values() {
        let entries = vec![
            json!({"level": "ERROR", "message": "fail"}),
            json!({"level": "INFO", "message": "ok"}),
            json!({"level": "ERROR", "message": "crash"}),
        ];

        let results =
            filter_values(entries.iter(), "select(.level == \"ERROR\")")
                .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["message"], "fail");
        assert_eq!(results[1]["message"], "crash");
    }

    #[test]
    fn test_transform_values() {
        let entries = vec![
            json!({"level": "ERROR", "message": "fail", "ts": 1}),
            json!({"level": "INFO", "message": "ok", "ts": 2}),
        ];

        let results =
            transform_values(entries.iter(), "{level, message}").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], json!({"level": "ERROR", "message": "fail"}));
        assert_eq!(results[1], json!({"level": "INFO", "message": "ok"}));
    }

    #[test]
    fn test_filter_array_any() {
        let filter =
            JqFilter::compile("select(.tags | any(. == \"testing\"))").unwrap();

        let match_entry = json!({"tags": ["testing", "debug"]});
        let no_match = json!({"tags": ["production"]});

        assert!(filter.matches(&match_entry));
        assert!(!filter.matches(&no_match));
    }

    #[test]
    fn test_run_identity() {
        let filter = JqFilter::compile(".").unwrap();
        let input = json!({"a": 1, "b": 2});
        let results = filter.run(&input);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_ref().unwrap(), &input);
    }
}
