use mejiro_engine::ffi::CState;
use mejiro_engine::{InputMode, Key, Session, State, translate};

fn enter(mode: InputMode) -> State {
    let raw = match mode {
        InputMode::Left => "#STKNYIAUntk-",
        InputMode::Right => "-STKNYIAUntk*",
        _ => "#STKNYIAUntk-STKNYIAUntk*",
    };
    let t = translate(raw, State::default(), true).unwrap();
    assert!(t.output.is_empty());
    assert_eq!(t.next.input_mode, mode);
    t.next
}

#[test]
fn phonetics_abbreviations_and_verbs_are_symmetric() {
    for mode in [InputMode::Left, InputMode::Right] {
        let state = enter(mode);
        for (keys, expected) in [
            ("KA", "か"),
            ("KAn", "かん"),
            ("KYIk", "くぃ"),
            ("K*", "この"),
            ("S*", "その"),
            ("ST*", "から"),
            ("KAU*", "こと"),
            ("TAU*", "ところ"),
            ("TKA*", "はなし"),
            ("KAUntk*", "ことは"),
            ("KAUn*", "こと、"),
            ("*", "する"),
            ("n*", "しない"),
            ("t*", "した"),
            ("k*", "します"),
            ("nt*", "しなかった"),
            ("nk*", "しません"),
            ("tk*", "しました"),
            ("ntk*", "して"),
            ("KU*", "くる"),
            ("KUt*", "きた"),
            ("TN*", "です"),
            ("TNt*", "でした"),
            ("K", "く"),
            ("Kt", "いた"),
            ("KI", "き"),
            ("KIA", "け"),
            ("KIn", "きん"),
            ("KIAt", "けつ"),
            ("KI*", "とき"),
            ("KIA*", "ける"),
            ("TIA*", "てる"),
            ("TIAn*", "てない"),
            ("TIAt*", "てた"),
            ("TIAk*", "てます"),
            ("TIAntk*", "てて"),
        ] {
            let raw = if mode == InputMode::Right {
                format!("-{keys}")
            } else {
                if let Some(base) = keys.strip_suffix('*') {
                    format!("#{base}-")
                } else {
                    keys.to_owned()
                }
            };
            assert_eq!(
                translate(&raw, state, true).unwrap().output.text.as_str(),
                expected,
                "{mode:?} {raw}"
            );
        }
        for (keys, expected) in [
            ("t", "、"),
            ("k", "。"),
            ("tk", "・"),
            ("nt", "？"),
            ("nk", "を"),
        ] {
            let raw = if mode == InputMode::Right {
                format!("-{keys}")
            } else {
                if let Some(base) = keys.strip_suffix('*') {
                    format!("#{base}-")
                } else {
                    keys.to_owned()
                }
            };
            assert_eq!(
                translate(&raw, state, true).unwrap().output.text.as_str(),
                expected
            );
        }
        for (keys, key) in [("n", Key::Space), ("ntk", Key::Return)] {
            let raw = if mode == InputMode::Right {
                format!("-{keys}")
            } else {
                if let Some(base) = keys.strip_suffix('*') {
                    format!("#{base}-")
                } else {
                    keys.to_owned()
                }
            };
            assert_eq!(
                translate(&raw, state, true).unwrap().output.operations()[0].key,
                key
            );
        }
    }
}

#[test]
fn switching_fallback_and_ffi_preserve_linguistic_state() {
    let mut s = Session::<20>::new();
    s.process("#STKNYIAUntk-").unwrap();
    assert_eq!(s.history_len(), 0);
    let event = s.process("KAtk-").unwrap();
    s.acknowledge(event.ticket.unwrap(), 2).unwrap();
    s.process("-STKNYIAUntk*").unwrap();
    assert_eq!(s.state().input_mode, InputMode::Right);
    assert_eq!(s.history_len(), 1);
    let packed: CState = s.state().into();
    assert_eq!(State::try_from(packed).unwrap(), s.state());
    let before = s.state();
    let probe = s.preview("KA-TA", true).unwrap();
    assert_eq!(s.state(), before);
    assert_eq!(probe.next.input_mode, InputMode::Both);
    assert_eq!(probe.output.text.as_str(), "かた");
    s.process("#STKNYIAUntk-STKNYIAUntk*").unwrap();
    assert_eq!(s.state().input_mode, InputMode::Both);
    s.process("-U").unwrap();
    assert_eq!(s.state().input_mode, InputMode::Both);
    assert_eq!(s.history_len(), 0);
}

#[test]
fn i_and_ia_without_star_use_shared_phonetics_for_every_consonant() {
    for mode in [InputMode::Left, InputMode::Right] {
        let state = enter(mode);
        for c in [
            "", "S", "T", "K", "N", "ST", "SK", "SN", "TK", "TN", "KN", "STK", "STN", "SKN", "TKN",
            "STKN",
        ] {
            for v in ["I", "IA"] {
                for p in ["", "n", "t", "k", "nt", "nk", "tk", "ntk"] {
                    let raw = if mode == InputMode::Left {
                        format!("{c}{v}{p}-")
                    } else {
                        format!("-{c}{v}{p}")
                    };
                    let reference =
                        translate(&format!("{c}{v}{p}-"), State::default(), false).unwrap();
                    let actual = translate(&raw, state, true).unwrap();
                    assert_eq!(actual.output, reference.output, "{mode:?}: {raw}");
                }
            }
        }
    }
}

#[test]
fn one_hand_undo_restores_state_and_backspace_is_a_history_boundary() {
    use mejiro_engine::{EventKind, Request};
    for (switch, kana, undo, backspace) in [
        ("#STKNYIAUntk-", "KI-", "#U-", "#AU-"),
        ("-STKNYIAUntk*", "-KI", "-U*", "-AU*"),
    ] {
        let mut session = Session::<20>::new();
        session.process(switch).unwrap();
        let before = session.state();
        let event = session.process(kana).unwrap();
        session.acknowledge(event.ticket.unwrap(), 1).unwrap();
        let preview = session.preview(undo, true).unwrap();
        assert_eq!(preview.request, Request::Undo);
        assert_eq!(preview.next, session.state());
        let event = session.process(undo).unwrap();
        assert_eq!(event.kind, EventKind::Undo);
        assert_eq!(event.erase_units, 1);
        assert_eq!(session.state(), before);
        let event = session.process(undo).unwrap();
        assert_eq!(event.erase_units, 2);
        assert_eq!(session.state().input_mode, before.input_mode);
        let event = session.process(kana).unwrap();
        session.acknowledge(event.ticket.unwrap(), 1).unwrap();
        let vowel = session.state().last_vowel();
        let event = session.process(backspace).unwrap();
        assert_eq!(event.output.operations()[0].key, Key::Backspace);
        assert_eq!(session.state().input_mode, before.input_mode);
        assert_eq!(session.state().last_vowel(), vowel);
        assert_eq!(session.history_len(), 0);
    }
    assert_ne!(
        translate("-U*", State::default(), true).unwrap().request,
        Request::Undo
    );
}
