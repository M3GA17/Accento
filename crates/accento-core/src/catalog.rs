#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Italian,
    French,
    Spanish,
    German,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::Italian => "it",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::German => "de",
        }
    }
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "it" => Some(Self::Italian),
            "fr" => Some(Self::French),
            "es" => Some(Self::Spanish),
            "de" => Some(Self::German),
            _ => None,
        }
    }
}

/// Original, deliberately bounded language sets. Order follows the user's language selection.
pub fn candidates(base: char, uppercase: bool, languages: &[Language]) -> Vec<char> {
    let mut result = Vec::with_capacity(12);
    for language in languages {
        let letters = match (language, base) {
            (Language::Italian, 'a') => "à",
            (Language::Italian, 'e') => "èé",
            (Language::Italian, 'i') => "ì",
            (Language::Italian, 'o') => "òó",
            (Language::Italian, 'u') => "ù",
            (Language::French, 'a') => "àâæ",
            (Language::French, 'c') => "ç",
            (Language::French, 'e') => "éèêë",
            (Language::French, 'i') => "îï",
            (Language::French, 'o') => "ôœ",
            (Language::French, 'u') => "ùûü",
            (Language::French, 'y') => "ÿ",
            (Language::Spanish, 'a') => "á",
            (Language::Spanish, 'e') => "é",
            (Language::Spanish, 'i') => "í",
            (Language::Spanish, 'n') => "ñ",
            (Language::Spanish, 'o') => "ó",
            (Language::Spanish, 'u') => "úü",
            (Language::German, 'a') => "ä",
            (Language::German, 'o') => "ö",
            (Language::German, 'u') => "ü",
            (Language::German, 's') => "ß",
            _ => "",
        };
        for mut character in letters.chars() {
            if uppercase {
                character = if character == 'ß' {
                    'ẞ'
                } else {
                    character.to_uppercase().next().unwrap_or(character)
                };
            }
            if !result.contains(&character) {
                result.push(character);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combined_sets_preserve_order_without_duplicates() {
        assert_eq!(
            candidates(
                'e',
                false,
                &[Language::Italian, Language::French, Language::Spanish]
            ),
            vec!['è', 'é', 'ê', 'ë']
        );
    }
    #[test]
    fn uppercase_is_one_precomposed_character() {
        assert_eq!(candidates('s', true, &[Language::German]), vec!['ẞ']);
        assert_eq!(candidates('e', true, &[Language::Italian]), vec!['È', 'É']);
    }
}
