const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 100;

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
        let search = Self::validate_text(term, Some(1), Some(100))?;
        let escaped = Self::escape_search_pattern(&search);
        Ok(SearchValidation {
            normalized: search,
            escaped,
        })
    }

    pub fn validate_text(
        term: String,
        min_length: Option<usize>,
        max_length: Option<usize>,
    ) -> Result<String, ValidationError> {
        let search = term.trim().to_owned();
        let length = search.chars().count();
        if search.is_empty() {
            return Err(ValidationError::Empty);
        } else if length < min_length.unwrap_or(MIN_LENGTH) {
            return Err(ValidationError::TooLong);
        } else if length > max_length.unwrap_or(MAX_LENGTH) {
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
    TooShort,
    TooLong,
}
impl ValidationError {
    pub fn message(&self) -> String {
        match self {
            ValidationError::Empty => "is empty",
            ValidationError::TooShort => "is too short",
            ValidationError::TooLong => "is too long",
        }
        .to_owned()
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
            Validator::validate_search_term("  Alice  ".to_string())
                .unwrap()
                .normalized,
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
        assert!(Validator::validate_search_term("é".repeat(100)).is_ok());
        assert!(matches!(
            Validator::validate_search_term("é".repeat(101)),
            Err(ValidationError::TooLong)
        ));
    }
}
