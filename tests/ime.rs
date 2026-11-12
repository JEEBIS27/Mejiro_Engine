use mejiro_engine::{
    ime::{SegmentKind, VerbClass, analyze},
    *,
};

#[test]
fn four_ambiguous_sentences_keep_explicit_information() {
    for (first, verb, particle, class) in [
        ("KYI-TKA", "I-Kt", false, VerbClass::Iku),
        ("KYI-t", "I-Kt", true, VerbClass::Iku),
        ("KYI-TKA", "I-SKt", false, VerbClass::GodanW),
        ("KYI-t", "I-SKt", true, VerbClass::GodanW),
    ] {
        let mut state = State::default();
        let mut reading = String::new();
        let mut particles = Vec::new();
        let mut classes = Vec::new();
        for raw in [first, "I-SYA", "t-", verb] {
            let a = analyze(raw, state, true, &RuleStore::new()).unwrap();
            for s in a.segments() {
                if s.kind == SegmentKind::Particle {
                    particles.push(a.canonical_text.as_str()[s.start..s.end].to_owned());
                }
                if let Some(v) = s.verb {
                    classes.push(v.class.unwrap());
                    assert_eq!(v.stem_bytes, Some(3));
                }
            }
            reading.push_str(a.canonical_text.as_str());
            state = a.translation.next;
        }
        assert_eq!(reading, "きょうはいしゃにいった");
        assert_eq!(
            particles,
            if particle {
                vec!["は", "に"]
            } else {
                vec!["に"]
            }
        );
        assert_eq!(classes, vec![class]);
    }
}

#[test]
fn observed_translation_preserves_existing_outputs_states_and_requests() {
    let rules = RuleStore::new();
    for input_mode in [InputMode::Both, InputMode::Left, InputMode::Right] {
        for mode in [Mode::Kana, Mode::Romaji, Mode::Jis] {
            for raw in [
                "KA",
                "KYI-t",
                "I-SKt",
                "K-",
                "KAU-",
                "k-k",
                "n-*",
                "#",
                "-U",
                "#-n",
                "#STKNYIAUntk-",
                "-STKNYIAUntk*",
                "KAUntk*",
                "-KAUntk*",
                "#KAUntk-",
                "TNt*",
                "-TNt*",
                "#TNt-",
            ] {
                let mut state = State::default();
                state.input_mode = input_mode;
                state.mode = mode;
                let ordinary = translate_with_rules(raw, state, true, &rules);
                let observed = analyze(raw, state, true, &rules);
                match (ordinary, observed) {
                    (Ok(t), Ok(a)) => {
                        assert_eq!(t.next, a.translation.next, "{raw} {mode:?} {input_mode:?}");
                        assert_eq!(t.request, a.translation.request);
                        assert_eq!(t.output, a.translation.output);
                        assert_eq!(t.dynamic_particle, a.translation.dynamic_particle);
                    }
                    (Err(e), Err(f)) => assert_eq!(e, f),
                    (a, b) => panic!("{raw} {mode:?} {input_mode:?}: {a:?} vs {b:?}"),
                }
            }
        }
    }
}

#[test]
fn canonical_reading_is_independent_of_output_encoding() {
    for mode in [Mode::Kana, Mode::Romaji, Mode::Jis] {
        let a = analyze(
            "I-Kt",
            {
                let mut state = State::default();
                state.mode = mode;
                state
            },
            true,
            &RuleStore::new(),
        )
        .unwrap();
        assert_eq!(a.canonical_text.as_str(), "いった");
        assert_eq!(a.segments()[0].verb.unwrap().class, Some(VerbClass::Iku));
    }
}

#[test]
fn grammar_branch_matrix_preserves_translation() {
    let rules = RuleStore::new();
    for mode in [InputMode::Both, InputMode::Left, InputMode::Right] {
        for consonant in ["", "K", "S", "ST", "TK", "TN", "SK", "STN"] {
            for vowel in ["", "A", "I", "IA", "U", "YI", "IU"] {
                for particle in ["", "n", "t", "k", "nt", "nk", "tk", "ntk"] {
                    for star in ["", "*"] {
                        let raw = match mode {
                            InputMode::Both => format!("I-{consonant}{vowel}{particle}{star}"),
                            InputMode::Left if star == "*" => {
                                format!("#{consonant}{vowel}{particle}-")
                            }
                            InputMode::Left => format!("{consonant}{vowel}{particle}"),
                            InputMode::Right => format!("-{consonant}{vowel}{particle}{star}"),
                        };
                        let mut state = State::default();
                        state.input_mode = mode;
                        let t = translate_with_rules(&raw, state, true, &rules);
                        let a = analyze(&raw, state, true, &rules);
                        match (t, a) {
                            (Ok(t), Ok(a)) => {
                                assert_eq!(t.output, a.translation.output);
                                assert_eq!(t.next, a.translation.next);
                            }
                            (Err(t), Err(a)) => assert_eq!(t, a),
                            (t, a) => panic!("{mode:?} {raw}: {t:?} vs {a:?}"),
                        }
                    }
                }
            }
        }
    }
}
