include!("catalog_generated.rs");

/// Unicode text candidates, stable order and no duplicates across selected languages.
/// Digraphs are typed normally; source coverage and optional extensions live in localization/.
pub fn candidates(
    base: char,
    uppercase: bool,
    languages: &[Language],
    currency_symbols: bool,
) -> Vec<String> {
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
    // Symbols follow language variants, preserving the usual first selection.
    // Currency symbols have no case and do not depend on the selected languages.
    if currency_symbols
        && let Some((_, symbols)) = CURRENCY_MAPS.iter().find(|(key, _)| *key == base)
    {
        for symbol in *symbols {
            if !result.iter().any(|value| value == symbol) {
                result.push((*symbol).to_owned());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn currencies_follow_accents_in_every_language_and_do_not_change_case() {
        assert_eq!(
            candidates('e', false, &[Language::Italian], true),
            ["è", "é", "€"]
        );
        assert_eq!(
            candidates('e', true, &[Language::Italian], true),
            ["È", "É", "€"]
        );
        for language in Language::ALL {
            for uppercase in [false, true] {
                let original = candidates('e', uppercase, &[*language], false);
                let extended = candidates('e', uppercase, &[*language], true);
                assert!(extended.starts_with(&original), "{}", language.code());
                assert_eq!(extended.iter().filter(|value| *value == "€").count(), 1);
            }
        }
        assert_eq!(candidates('l', false, &[Language::Italian], true), ["£"]);
        assert!(candidates('l', false, &[Language::Italian], false).is_empty());
        assert_eq!(candidates('y', true, &[Language::Italian], true), ["¥"]);
    }
    #[test]
    fn currency_data_matches_the_shared_registry_and_combined_lists_have_no_duplicates() {
        let data: std::collections::BTreeMap<String, Vec<String>> =
            serde_json::from_str(include_str!("../../../localization/currencies.json")).unwrap();
        assert_eq!(data.len(), CURRENCY_MAPS.len());
        for &(base, symbols) in CURRENCY_MAPS {
            assert_eq!(data[&base.to_string()], symbols);
            for uppercase in [false, true] {
                let list = candidates(base, uppercase, Language::ALL, true);
                for symbol in symbols {
                    assert_eq!(
                        list.iter()
                            .filter(|value| value.as_str() == *symbol)
                            .count(),
                        1
                    );
                }
                let unique: std::collections::HashSet<_> = list.iter().collect();
                assert_eq!(list.len(), unique.len());
            }
        }
    }
    #[test]
    fn combined_sets_preserve_order_without_duplicates() {
        assert_eq!(
            candidates(
                'e',
                false,
                &[Language::Italian, Language::French, Language::Spanish],
                false
            ),
            vec!["è", "é", "ê", "ë"]
        );
    }
    #[test]
    fn uppercase_is_one_precomposed_character() {
        assert_eq!(candidates('s', true, &[Language::German], false), vec!["ẞ"]);
        assert_eq!(
            candidates('e', true, &[Language::Italian], false),
            vec!["È", "É"]
        );
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
                candidates(base, false, &[language], false),
                lower.chars().map(|c| c.to_string()).collect::<Vec<_>>()
            );
            assert_eq!(
                candidates(base, true, &[language], false),
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
                    reachable.extend(candidates(base, false, &[*language], false));
                    reachable.extend(candidates(base, true, &[*language], false));
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
                candidates(base, false, &[Language::from_code(code).unwrap()], false)
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
                candidates(
                    base,
                    uppercase,
                    &[Language::from_code(code).unwrap()],
                    false
                )
                .iter()
                .any(|value| value == expected),
                "{code}: {expected}"
            );
        }
    }
}
