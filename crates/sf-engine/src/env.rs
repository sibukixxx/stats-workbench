use sf_core::{SfError, SfResult};

/// Variables that recipes may reference by exact name.
const ALLOWED_EXACT: &[&str] = &["DATABASE_URL"];
/// Any variable starting with this prefix may be referenced by recipes.
const ALLOWED_PREFIX: &str = "SF_";

fn is_allowed(name: &str) -> bool {
    name.starts_with(ALLOWED_PREFIX) || ALLOWED_EXACT.contains(&name)
}

/// Expand `${VAR_NAME}` patterns with environment variables.
///
/// Recipes are shared, executable documents, so expansion is deliberately
/// restricted: only `SF_*` variables and `DATABASE_URL` can be referenced.
/// Anything else (`${AWS_SECRET_ACCESS_KEY}`, `${HOME}`, ...) is rejected so a
/// recipe cannot smuggle the operator's secrets into a connection string,
/// a SQL query, or an output path. A referenced variable that is not set is
/// also an error rather than being silently passed through.
pub fn expand_env_vars(input: &str) -> SfResult<String> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            let mut terminated = false;
            for c in chars.by_ref() {
                if c == '}' {
                    terminated = true;
                    break;
                }
                var_name.push(c);
            }
            if !terminated {
                return Err(SfError::Validation(format!(
                    "Unterminated environment variable reference '${{{}'",
                    var_name
                )));
            }
            if !is_allowed(&var_name) {
                return Err(SfError::Validation(format!(
                    "Environment variable '${{{}}}' is not allowed in recipes (only SF_* and DATABASE_URL can be expanded)",
                    var_name
                )));
            }
            match std::env::var(&var_name) {
                Ok(val) => output.push_str(&val),
                Err(_) => {
                    return Err(SfError::Validation(format!(
                        "Environment variable '{}' referenced by the recipe is not set",
                        var_name
                    )));
                }
            }
        } else {
            output.push(ch);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_existing_var() {
        std::env::set_var("SF_TEST_EXPAND", "replaced");
        let result = expand_env_vars("prefix_${SF_TEST_EXPAND}_suffix").unwrap();
        assert_eq!(result, "prefix_replaced_suffix");
        std::env::remove_var("SF_TEST_EXPAND");
    }

    #[test]
    fn test_expand_missing_allowed_var_is_error() {
        let err = expand_env_vars("${SF_DEFINITELY_NOT_SET_XYZ}").unwrap_err();
        assert!(err.to_string().contains("not set"));
    }

    #[test]
    fn test_expand_disallowed_var_is_error() {
        std::env::set_var("NOT_FOR_RECIPES", "secret");
        let err = expand_env_vars("url = \"${NOT_FOR_RECIPES}\"").unwrap_err();
        assert!(err.to_string().contains("not allowed"));
        std::env::remove_var("NOT_FOR_RECIPES");
    }

    #[test]
    fn test_expand_unterminated_is_error() {
        assert!(expand_env_vars("${SF_OPEN").is_err());
    }

    #[test]
    fn test_expand_no_vars() {
        let result = expand_env_vars("no variables here").unwrap();
        assert_eq!(result, "no variables here");
    }
}
