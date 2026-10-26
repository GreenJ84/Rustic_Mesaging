
pub struct Validator;
impl Validator {
    fn escape_search_pattern(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            if matches!(character, '\\' | '%' | '_') {
                escaped.push('\\');
            }
            escaped.push(character);
        }
        escaped
    }

    pub fn validate_search_term(term: String) -> Result<SearchValidation, ValidationError> {
        let search = Self::validate_text(term, 100)?;
        let escaped = Self::escape_search_pattern(&search);
        Ok(SearchValidation { normalized: search, escaped })
    }

    pub fn validate_text(term: String, length_limit: usize) -> Result<String, ValidationError> {
        let search = term.trim().to_owned();
        if search.is_empty() {
            return Err(ValidationError::Empty);
        } else if search.chars().count() > length_limit {
            return Err(ValidationError::TooLong);
        }
        Ok(search)
    }
}

pub struct SearchValidation {
    pub normalized: String,
    pub escaped: String,
}

#[derive(Debug)]
pub enum ValidationError {
    Empty,
    TooLong,
}
impl ValidationError {
    pub fn message(&self) -> &str {
        match self {
            ValidationError::Empty => "Search term is empty",
            ValidationError::TooLong => "Search term too long",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ValidationError, Validator};

    #[test]
    fn escapes_like_wildcards_and_escape_characters() {
        assert_eq!(
            Validator::escape_search_pattern(r"50%_off\today"),
            r"50\%\_off\\today"
        );
    }

    #[test]
    fn leaves_plain_search_terms_unchanged() {
        assert_eq!(
            Validator::escape_search_pattern("rust server"),
            "rust server"
        );
    }

    #[test]
    fn normalizes_search_terms() {
        assert_eq!(
            Validator::validate_search_term("  Alice  ".to_string()).unwrap().normalized,
            "Alice"
        );
    }

    #[test]
    fn rejects_empty_search_terms() {
        assert!(matches!(
            Validator::validate_search_term(" \t\n ".to_string()),
            Err(ValidationError::Empty)
        ));
    }

    #[test]
    fn limits_search_terms_by_character_count() {
        assert!(Validator::validate_search_term("é".repeat(125)).is_ok());
        assert!(matches!(
            Validator::validate_search_term("é".repeat(126)),
            Err(ValidationError::TooLong)
        ));
    }
}