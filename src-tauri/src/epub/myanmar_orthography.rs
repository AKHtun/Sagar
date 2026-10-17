//! Myanmar Unicode canonical sequence normalizer for Sagar.
//!
//! Enforces Unicode 5.2+ canonical combining character sequences to eliminate
//! dotted-circle (◌) rendering glitches on e-readers (Kindle, Kobo, Boox) and
//! web views caused by LLM-generated out-of-order diacritics.

/// Canonical combining sequence:
/// Consonant/Independent Vowel -> Medials (ya, ra, wa, ha) -> Upper Vowel -> Lower Vowel -> Anusvara -> Asat -> Tone/Visarga
pub fn normalize_myanmar_text(input: &str) -> String {
    if !input.chars().any(is_myanmar_char) {
        return input.to_string();
    }

    let mut result = input.to_string();

    // 1. Remove zero-width non-joiners / zero-width spaces before e-vowel or between diacritics
    result = result.replace("\u{200c}\u{1031}", "\u{1031}");
    result = result.replace("\u{200b}", "");

    // 2. Reorder visual pre-base e-vowel (U+1031) if generated before the consonant
    // E.g. \u1031 + \u1000 -> \u1000 + \u1031 (ေက -> ကေ)
    result = reorder_visual_e_vowel(&result);

    // 3. Reorder visual pre-base medial-ra (U+103c) if generated before the consonant
    // E.g. \u103c + \u1000 -> \u1000 + \u103c (ြက -> ကြ)
    result = reorder_visual_medial_ra(&result);

    // 4. Inverted lower vowel (\u102f, \u1030) before upper vowel (\u102d, \u102e)
    // Canonical order: Upper Vowel (102d/102e) THEN Lower Vowel (102f/1030)
    result = fix_inverted_upper_lower_vowels(&result);

    // 5. Inverted anusvara (\u1036) before lower vowel (\u102f, \u1030)
    // Canonical order: Lower Vowel (102f/1030) THEN Anusvara (1036)
    result = fix_inverted_anusvara_lower_vowels(&result);

    // 6. Deduplicate repeated identical diacritics (frequent hallucination artifact)
    result = deduplicate_diacritics(&result);

    // 7. Canonical historical kingship names (King Sawlu: စောလူး with sa-lone, never ဆောလူး)
    result = result.replace("ဆောလူး", "စောလူး");
    result = result.replace("ဆောလု", "စောလု");

    result
}

#[inline]
fn is_myanmar_char(c: char) -> bool {
    matches!(c, '\u{1000}'..='\u{109F}' | '\u{AA60}'..='\u{AA7F}')
}

#[inline]
fn is_myanmar_consonant(c: char) -> bool {
    matches!(c, '\u{1000}'..='\u{1021}' | '\u{1023}'..='\u{102A}' | '\u{103F}' | '\u{104E}')
}

#[inline]
fn is_myanmar_medial(c: char) -> bool {
    matches!(c, '\u{103B}'..='\u{103E}')
}

#[inline]
fn is_myanmar_consonant_or_medial(c: char) -> bool {
    is_myanmar_consonant(c) || is_myanmar_medial(c)
}

/// Reorders visual e-vowel (U+1031) preceding consonant to proper logical order
fn reorder_visual_e_vowel(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let is_visual_candidate = i == 0 || !is_myanmar_consonant_or_medial(chars[i - 1]);
        if is_visual_candidate
            && chars[i] == '\u{1031}'
            && i + 1 < chars.len()
            && is_myanmar_consonant(chars[i + 1])
        {
            let cons = chars[i + 1];
            out.push(cons);
            i += 2;
            // Collect any attached medials
            while i < chars.len() && is_myanmar_medial(chars[i]) {
                out.push(chars[i]);
                i += 1;
            }
            out.push('\u{1031}');
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Reorders visual medial-ra (U+103C) preceding consonant
fn reorder_visual_medial_ra(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let is_visual_candidate = i == 0 || !is_myanmar_consonant_or_medial(chars[i - 1]);
        if is_visual_candidate
            && chars[i] == '\u{103C}'
            && i + 1 < chars.len()
            && is_myanmar_consonant(chars[i + 1])
        {
            out.push(chars[i + 1]);
            out.push('\u{103C}');
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Upper vowel (\u102d, \u102e) must precede lower vowel (\u102f, \u1030)
fn fix_inverted_upper_lower_vowels(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len()
            && matches!(chars[i], '\u{102F}' | '\u{1030}')
            && matches!(chars[i + 1], '\u{102D}' | '\u{102E}')
        {
            out.push(chars[i + 1]);
            out.push(chars[i]);
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Lower vowel (\u102f, \u1030) must precede anusvara (\u1036)
fn fix_inverted_anusvara_lower_vowels(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len()
            && chars[i] == '\u{1036}'
            && matches!(chars[i + 1], '\u{102F}' | '\u{1030}')
        {
            out.push(chars[i + 1]);
            out.push(chars[i]);
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Strips duplicate adjacent combining marks
fn deduplicate_diacritics(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev: Option<char> = None;
    for c in text.chars() {
        if let Some(p) = prev {
            if p == c && matches!(c, '\u{102B}'..='\u{103E}') {
                continue; // skip duplicate diacritic
            }
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_e_vowel_reordering() {
        // Visual: ေ (1031) + က (1000) -> Logical: က (1000) + ေ (1031)
        let visual = "\u{1031}\u{1000}";
        let expected = "\u{1000}\u{1031}";
        assert_eq!(normalize_myanmar_text(visual), expected);
    }

    #[test]
    fn test_visual_e_vowel_with_medial() {
        // Visual: ေ (1031) + က (1000) + ျ (103B) -> Logical: က (1000) + ျ (103B) + ေ (1031)
        let visual = "\u{1031}\u{1000}\u{103B}";
        let expected = "\u{1000}\u{103B}\u{1031}";
        assert_eq!(normalize_myanmar_text(visual), expected);
    }

    #[test]
    fn test_inverted_vowels_reordering() {
        // Inverted: ု (102F) + ိ (102D) -> Logical: ိ (102D) + ု (102F)
        let inverted = "က\u{102F}\u{102D}";
        let expected = "က\u{102D}\u{102F}";
        assert_eq!(normalize_myanmar_text(inverted), expected);
    }

    #[test]
    fn test_inverted_anusvara() {
        // Inverted: ံ (1036) + ု (102F) -> Logical: ု (102F) + ံ (1036)
        let inverted = "က\u{1036}\u{102F}";
        let expected = "က\u{102F}\u{1036}";
        assert_eq!(normalize_myanmar_text(inverted), expected);
    }

    #[test]
    fn test_deduplicate_diacritics() {
        // Hallucinated duplicate ာ (102C)
        let dup = "က\u{102C}\u{102C}";
        let expected = "က\u{102C}";
        assert_eq!(normalize_myanmar_text(dup), expected);
    }
}
