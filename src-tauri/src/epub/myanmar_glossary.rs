//! Sovereign Myanmar Technical & Domain Glossary Engine for WEcoS and Sagar.
//!
//! Provides curated domain terminologies across Engineering, Tribology,
//! Materials Science, Laboratory Analysis, and Computing. When translating chapters into
//! Myanmar, matching English terms are dynamically surfaced and injected into the AI
//! prompt as mandatory term mappings to eliminate hallucinated, non-standard, or slop translations.

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct GlossaryEntry {
    pub english: &'static str,
    pub myanmar: &'static str,
    pub domain: &'static str,
}

pub static GLOSSARY: &[GlossaryEntry] = &[
    // --- Executive, Reports & Laboratory Analysis ---
    GlossaryEntry {
        english: "executive summary",
        myanmar: "အစီရင်ခံစာအကျဉ်းချုပ်",
        domain: "Business",
    },
    GlossaryEntry {
        english: "report of analysis",
        myanmar: "ဓာတ်ခွဲစမ်းသပ်စစ်ဆေးချက် အစီရင်ခံစာ",
        domain: "Laboratory",
    },
    GlossaryEntry {
        english: "laboratory report",
        myanmar: "ဓာတ်ခွဲခန်း အစီရင်ခံစာ",
        domain: "Laboratory",
    },
    GlossaryEntry {
        english: "sample id",
        myanmar: "နမူနာအမှတ်",
        domain: "Laboratory",
    },
    GlossaryEntry {
        english: "lab reference number",
        myanmar: "ဓာတ်ခွဲခန်း ကိုးကားအမှတ်",
        domain: "Laboratory",
    },
    GlossaryEntry {
        english: "date reported",
        myanmar: "အစီရင်ခံသည့် ရက်စွဲ",
        domain: "Laboratory",
    },
    GlossaryEntry {
        english: "date received",
        myanmar: "လက်ခံရရှိသည့် ရက်စွဲ",
        domain: "Laboratory",
    },
    GlossaryEntry {
        english: "disclaimer",
        myanmar: "တာဝန်ယူမှုဆိုင်ရာ ရှင်းလင်းချက်",
        domain: "Legal",
    },
    GlossaryEntry {
        english: "terms and conditions",
        myanmar: "စည်းကမ်းနှင့် သတ်မှတ်ချက်များ",
        domain: "Legal",
    },
    GlossaryEntry {
        english: "observation",
        myanmar: "လေ့လာတွေ့ရှိချက်",
        domain: "Science",
    },
    GlossaryEntry {
        english: "recommendation",
        myanmar: "အကြံပြုချက်",
        domain: "Science",
    },
    GlossaryEntry {
        english: "conclusion",
        myanmar: "နိဂုံး / အနှစ်ချုပ်",
        domain: "Science",
    },
    GlossaryEntry {
        english: "spec limit",
        myanmar: "စံသတ်မှတ် ကန့်သတ်ချက်",
        domain: "Laboratory",
    },

    // --- Engineering, Tribology & Machinery ---
    GlossaryEntry {
        english: "viscosity",
        myanmar: "စေးပျစ်မှု (Viscosity)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "acid number",
        myanmar: "အက်ဆစ်တန်ဖိုး (Acid Number)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "flash point",
        myanmar: "မီးတောက်အပူချိန် (Flash Point)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "water content",
        myanmar: "ရေပါဝင်မှု (Water Content)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "wear metals",
        myanmar: "ပွန်းစားသတ္တုများ (Wear Metals)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "particle count",
        myanmar: "အမှုန်ရေတွက်မှု (Particle Count)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "cleanliness code",
        myanmar: "သန့်စင်မှုအဆင့် သတ်မှတ်ချက် (Cleanliness Code)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "foaming tendency",
        myanmar: "အမြှုပ်ထွက်နိုင်စွမ်း (Foaming Tendency)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "foam stability",
        myanmar: "အမြှုပ်တည်ငြိမ်မှု (Foam Stability)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "air release time",
        myanmar: "လေထုတ်လွှတ်ချိန် (Air Release Time)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "gas turbine",
        myanmar: "ဓာတ်ငွေ့တာဘိုင် (Gas Turbine)",
        domain: "Machinery",
    },
    GlossaryEntry {
        english: "lubricant",
        myanmar: "ချောဆီ (Lubricant)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "tribology",
        myanmar: "ထရိုင်ဘိုလော်ဂျီ (ပွတ်တိုက်မှုနှင့် ချောဆီဆိုင်ရာ သိပ္ပံ)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "contamination",
        myanmar: "အညစ်အကြေး ရောနှောမှု (Contamination)",
        domain: "Tribology",
    },
    GlossaryEntry {
        english: "oxidation",
        myanmar: "အောက်ဆီဒေးရှင်း ဖြစ်စဉ် (Oxidation)",
        domain: "Tribology",
    },

    // --- Computing & AI Observability ---
    GlossaryEntry {
        english: "observability",
        myanmar: "စနစ်လေ့လာစောင့်ကြည့်နိုင်မှု (Observability)",
        domain: "Computing",
    },
    GlossaryEntry {
        english: "telemetry",
        myanmar: "တယ်လီမီထရီ အချက်အလက် (Telemetry)",
        domain: "Computing",
    },
    GlossaryEntry {
        english: "latency",
        myanmar: "တုံ့ပြန်မှု ကြန့်ကြာချိန် (Latency)",
        domain: "Computing",
    },
    GlossaryEntry {
        english: "hallucination",
        myanmar: "မှားယွင်းတီထွင်မှု (Hallucination)",
        domain: "Computing",
    },
    GlossaryEntry {
        english: "architecture",
        myanmar: "တည်ဆောက်ပုံစနစ် (Architecture)",
        domain: "Computing",
    },
];

/// Scans the given HTML/text for occurrences of glossary terms (case-insensitively)
/// and returns a list of matched terms with their standard Myanmar translations.
pub fn find_matching_terms(text: &str) -> BTreeMap<&'static str, &'static str> {
    let lower_text = text.to_lowercase();
    let mut matched = BTreeMap::new();

    for entry in GLOSSARY {
        if lower_text.contains(entry.english) {
            matched.insert(entry.english, entry.myanmar);
        }
    }

    matched
}

/// Formats matching glossary entries into a clean instruction block for the AI prompt.
pub fn build_glossary_prompt_section(text: &str) -> Option<String> {
    let matches = find_matching_terms(text);
    if matches.is_empty() {
        return None;
    }

    let mut section = String::from("\n          Mandatory Technical Terminology Mapping:\n");
    for (en, my) in matches {
        section.push_str(&format!("          - \"{}\" -> \"{}\"\n", en, my));
    }
    section.push_str("          Strictly adhere to the term mappings above for technical consistency.\n");

    Some(section)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_found_in_technical_html() {
        let html = "<h1>Report of Analysis</h1><p>Measuring Viscosity and Air Release Time of Mobil DTE 746</p>";
        let matched = find_matching_terms(html);
        assert!(matched.contains_key("report of analysis"));
        assert!(matched.contains_key("viscosity"));
        assert!(matched.contains_key("air release time"));
        assert_eq!(matched["report of analysis"], "ဓာတ်ခွဲစမ်းသပ်စစ်ဆေးချက် အစီရင်ခံစာ");
    }

    #[test]
    fn format_glossary_section() {
        let html = "<p>The executive summary highlights lubricant viscosity.</p>";
        let section = build_glossary_prompt_section(html).expect("has matches");
        assert!(section.contains("Mandatory Technical Terminology Mapping:"));
        assert!(section.contains("executive summary"));
        assert!(section.contains("အစီရင်ခံစာအကျဉ်းချုပ်"));
        assert!(section.contains("viscosity"));
        assert!(section.contains("စေးပျစ်မှု (Viscosity)"));
    }

    #[test]
    fn empty_when_no_technical_terms() {
        let html = "<p>The cat sat on the mat.</p>";
        assert!(build_glossary_prompt_section(html).is_none());
    }
}
