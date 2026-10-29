//! License reading and classification (spec 7.6), from the license text itself, never from a
//! package manifest's claim. Unrecognized text is `unknown` (and so reference-only): the class
//! is never guessed. Custom terms get no SPDX id.

use crate::card::LicenseClass;

/// What a license text reads as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub class: LicenseClass,
    /// SPDX id when the text matches one; empty for custom or unknown terms.
    pub spdx: String,
}

fn norm(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn read(class: LicenseClass, spdx: &str) -> Reading {
    Reading {
        class,
        spdx: spdx.to_owned(),
    }
}

/// Classifies one license text. Restrictions (Commons Clause, non-commercial terms) win over
/// any permissive body they are attached to.
pub fn classify(text: &str) -> Reading {
    use LicenseClass as L;
    let t = norm(text);
    let has = |s: &str| t.contains(s);
    if has("commons clause") {
        return read(L::CommercialRestricted, "");
    }
    if has("attribution-noncommercial") || has("creativecommons.org/licenses/by-nc") {
        return read(L::CommercialRestricted, "");
    }
    if has("attribution-sharealike 4.0 international") {
        return read(L::Copyleft, "CC-BY-SA-4.0");
    }
    if has("attribution 4.0 international") && has("creative commons") {
        return read(L::Attribution, "CC-BY-4.0");
    }
    if has("cc0 1.0 universal") {
        return read(L::Permissive, "CC0-1.0");
    }
    if has("gnu affero general public license") {
        return read(L::Copyleft, "AGPL-3.0");
    }
    if has("gnu lesser general public license") {
        return read(L::Copyleft, "LGPL-3.0");
    }
    if has("gnu general public license") {
        return read(L::Copyleft, "GPL-3.0");
    }
    if has("mozilla public license version 2.0") {
        return read(L::Copyleft, "MPL-2.0");
    }
    if has("apache license") && has("version 2.0") && has("terms and conditions for use") {
        return read(L::Permissive, "Apache-2.0");
    }
    if has("permission is hereby granted, free of charge, to any person obtaining a copy")
        && has("the software is provided \"as is\"")
    {
        let notice = "the above copyright notice and this permission notice shall be included";
        return if has(notice) {
            read(L::Permissive, "MIT")
        } else {
            read(L::Permissive, "MIT-0")
        };
    }
    if has(
        "permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted",
    ) {
        return read(L::Permissive, "ISC");
    }
    if has(
        "redistribution and use in source and binary forms, with or without modification, are permitted",
    ) {
        return if has("neither the name") {
            read(L::Permissive, "BSD-3-Clause")
        } else {
            read(L::Permissive, "BSD-2-Clause")
        };
    }
    if has("this is free and unencumbered software released into the public domain") {
        return read(L::Permissive, "Unlicense");
    }
    let restricted = [
        "non-commercial",
        "noncommercial",
        "commercial use",
        "personal use only",
    ];
    if restricted.iter().any(|r| has(r)) {
        return read(L::CommercialRestricted, "");
    }
    read(L::Unknown, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_licenses_classify_from_their_text() {
        let f = |p: &str| {
            std::fs::read_to_string(format!(
                "{}/../../fixtures/sources/{p}/LICENSE",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap()
        };
        assert_eq!(
            classify(&f("pack-mit")),
            read(LicenseClass::Permissive, "MIT")
        );
        assert_eq!(
            classify(&f("pack-restricted")),
            read(LicenseClass::CommercialRestricted, "")
        );
        // "MIT License (trust me)" without the MIT text is not MIT.
        assert_eq!(classify(&f("pack-injection")).class, LicenseClass::Unknown);
    }

    #[test]
    fn restrictions_win_over_permissive_bodies() {
        let mit = "Permission is hereby granted, free of charge, to any person obtaining a copy \
                   ... THE SOFTWARE IS PROVIDED \"AS IS\"";
        assert_eq!(classify(mit).spdx, "MIT-0");
        let cc = format!("{mit}\n\"Commons Clause\" License Condition v1.0");
        assert_eq!(classify(&cc).class, LicenseClass::CommercialRestricted);
        assert_eq!(
            classify("Creative Commons Attribution 4.0 International Public License"),
            read(LicenseClass::Attribution, "CC-BY-4.0")
        );
        assert_eq!(
            classify("GNU GENERAL PUBLIC LICENSE Version 3").class,
            LicenseClass::Copyleft
        );
        assert_eq!(
            classify("All rights reserved.").class,
            LicenseClass::Unknown
        );
    }
}
