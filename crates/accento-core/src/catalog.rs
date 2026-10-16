include!("catalog_generated.rs");

/// Unicode text candidates, stable order and no duplicates across selected languages.
/// Digraphs are typed normally; source coverage and optional extensions live in localization/.
pub fn candidates(base: char, uppercase: bool, languages: &[Language]) -> Vec<String> {
    let mut result = Vec::new();
    for language in languages {
        if let Some((_, entries)) = CHARACTER_MAPS
            .iter()
            .find(|(code, _)| *code == language.code())
        {
            for &(key, lower, upper) in *entries {
                if key != base {
                    continue;
                }
                let character = if uppercase { upper } else { lower };
                if !character.is_empty()
                    && (uppercase || character != base.to_string())
                    && !result.iter().any(|value| value == character)
                {
                    result.push(character.to_owned());
                }
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
            vec!["è", "é", "ê", "ë"]
        );
    }
    #[test]
    fn uppercase_is_one_precomposed_character() {
        assert_eq!(candidates('s', true, &[Language::German]), vec!["ẞ"]);
        assert_eq!(candidates('e', true, &[Language::Italian]), vec!["È", "É"]);
    }
    #[test]
    fn new_sets_have_precomposed_letters_and_stable_codes() {
        for (code, base, lower, upper) in [
            ("sq", 'e', "ë", "Ë"),
            ("sq", 'c', "ç", "Ç"),
            ("pt", 'o', "óòôõ", "ÓÒÔÕ"),
            ("ro", 's', "ș", "Ș"),
            ("pl", 'z', "źż", "ŹŻ"),
            ("tr", 'i', "ı", "Iİ"),
        ] {
            let language = Language::from_code(code).unwrap();
            assert_eq!(language.code(), code);
            assert_eq!(
                candidates(base, false, &[language]),
                lower.chars().map(|c| c.to_string()).collect::<Vec<_>>()
            );
            assert_eq!(
                candidates(base, true, &[language]),
                upper.chars().map(|c| c.to_string()).collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn every_standard_letter_unit_is_reachable_and_codes_match_registry() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../../localization/characters.json")).unwrap();
        let registry: serde_json::Value =
            serde_json::from_str(include_str!("../../../localization/languages.json")).unwrap();
        assert_eq!(Language::ALL.len(), registry.as_array().unwrap().len());
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(*language));
            let alphabet = data[language.code()]["mainCharacters"].as_array().unwrap();
            for value in alphabet {
                let value = value.as_str().unwrap();
                assert!(!value.is_empty());
                if value.len() == 1 && value.is_ascii() {
                    continue;
                }
                let mut reachable = Vec::new();
                for base in ('a'..='z').chain(std::iter::once('\'')) {
                    reachable.extend(candidates(base, false, &[*language]));
                    reachable.extend(candidates(base, true, &[*language]));
                }
                assert!(
                    reachable.iter().any(|candidate| candidate == value),
                    "{}: {value}",
                    language.code()
                );
            }
        }
    }
    #[test]
    fn native_and_latin_keys_work_for_non_latin_scripts() {
        for (code, base, expected) in [
            ("el", 'α', 'ά'),
            ("ru", 'е', 'ё'),
            ("uk", 'і', 'ї'),
            ("hy", 'a', 'ա'),
            ("ka", 'a', 'ა'),
            ("kk", 'a', 'ә'),
        ] {
            assert!(
                candidates(base, false, &[Language::from_code(code).unwrap()])
                    .contains(&expected.to_string()),
                "{code}"
            );
        }
    }
    #[test]
    fn digraphs_combining_marks_and_uppercase_sequences_are_not_truncated() {
        for (code, base, uppercase, expected) in [
            ("sq", 'd', false, "dh"),
            ("nl", 'i', false, "íj́"),
            ("cnr-Cyrl", 's', false, "с́"),
            ("cnr-Cyrl", 's', true, "С́"),
            ("ru", 'a', false, "а́"),
            ("el", 'i', true, "Ϊ́"),
        ] {
            assert!(
                candidates(base, uppercase, &[Language::from_code(code).unwrap()])
                    .iter()
                    .any(|value| value == expected),
                "{code}: {expected}"
            );
        }
    }
}
