use mejiro_engine::*;

fn preview(raw: &str, state: State) -> Translation {
    translate(raw, state, false).unwrap()
}

#[test]
fn revised_minor_vowels_and_exception_kana() {
    for (stroke, expected) in [
        ("Yk-", "うぁ"),
        ("YInk-", "うぃん"),
        ("YIAntk-", "うぇー"),
        ("YIAUtk-", "うぉっ"),
        ("IAUtk-", "いぇっ"),
        ("Yt-", "いい"),
        ("IUnt-", "おいん"),
        ("YIUnt-", "よいん"),
        ("IAUt-", "あう"),
        ("YIAUt-", "いぇい"),
        ("KYk-", "くぁ"),
        ("TYIAUk-", "つぉ"),
        ("TKYIAk-", "ふぇ"),
        ("STKNYk-", "ゔぁ"),
        ("STKNYIk-", "ゔぃ"),
        ("STKNYIAk-", "ゔぇ"),
        ("STKNYIAUntk-", "ゔぉー"),
        ("STKNU-", "ゔ"),
        ("STKNYU-", "ふゅ"),
        ("STKNIU-", "てゅ"),
        ("STKNYIUntk-", "ゔゅー"),
        ("TNIU-", "とぅ"),
        ("TNYIUtk-", "どぅっ"),
        ("TNYIUntk-", "どぅー"),
        ("TNYIAUntk-", "づぉー"),
        ("TNYA-", "てぃ"),
        ("TNYU-", "でゅ"),
        ("TNYAU-", "でぃ"),
        ("SKU-", "ゎ"),
        ("SKYA-", "ゐ"),
        ("SKYU-", "いう"),
        ("SKYAU-", "ゑ"),
        ("SKYk-", "うぁ"),
        ("SKIAUk-", "うぃぇ"),
        ("STNYk-", "ぅぁ"),
        // IU/YIU t/nt still select minor vowels, while n/tk/ntk extend exceptions.
        ("TNIUn-", "とぅん"),
        ("TNIUt-", "どい"),
        ("TNYIUnt-", "ぢょいん"),
    ] {
        assert_eq!(
            preview(stroke, State::default()).output.text.as_str(),
            expected,
            "{stroke}"
        );
    }
}
fn input<const H: usize>(s: &mut Session<H>, raw: &str) -> Event {
    let event = s.process(raw).unwrap();
    if let Some(ticket) = event.ticket {
        // Test host emits kana directly. Firmware must count its actual IME
        // input units; never copy this test host's character metric blindly.
        let units = event.output.text.as_str().chars().count() as u32
            + event.output.operations().len() as u32;
        if !event.output.is_empty() {
            s.acknowledge(ticket, units).unwrap();
        }
    }
    event
}

#[test]
fn grammar_uses_classified_output_not_suffix_guessing() {
    let mut s = Session::<20>::new();
    assert_eq!(input(&mut s, "n-*").output.text.as_str(), "している");
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "のが");
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "な");
    input(&mut s, "n-*");
    input(&mut s, "KA-STU");
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "な");
}
#[test]
fn he_ka_and_attached_particles() {
    for (word, expected) in [
        ("n-*", "か"),
        ("-TNk", "か"),
        ("K-T", "か"),
        ("K-KAU", "へ"),
        ("N-N", "か"),
        ("KA-", "へ"),
    ] {
        let mut s = Session::<20>::new();
        input(&mut s, word);
        assert_eq!(
            input(&mut s, "ntk-").output.text.as_str(),
            expected,
            "{word}"
        );
    }
    assert_eq!(
        preview("Kntk-Tk", State::default()).output.text.as_str(),
        "ことへの"
    );
    assert_eq!(
        preview("Nntk-Nk", State::default()).output.text.as_str(),
        "なにかの"
    );
}
#[test]
fn undo_restores_context_and_actual_receipt() {
    let mut s = Session::<20>::new();
    input(&mut s, "n-*");
    let e = s.process("k-k").unwrap();
    s.acknowledge(e.ticket.unwrap(), 17).unwrap();
    let e = s.process("-U").unwrap();
    assert_eq!(e.kind, EventKind::Undo);
    assert_eq!(e.erase_units, 17);
    assert_eq!(input(&mut s, "k-nk").output.text.as_str(), "のが、");
}
#[test]
fn undo_never_guesses_delete_counts() {
    let mut s = Session::<20>::new();
    let e = s.process("KA-").unwrap();
    let state = s.state();
    assert_eq!(s.process("-U").unwrap_err(), Error::Unacknowledged);
    assert_eq!(s.state(), state);
    assert_eq!(s.history_len(), 1);
    s.acknowledge(e.ticket.unwrap(), 2).unwrap();
    assert_eq!(s.process("-U").unwrap().erase_units, 2);
}
#[test]
fn undo_without_history_erases_two_host_units() {
    let mut s = Session::<1>::new();
    let empty = s.process("-U").unwrap();
    assert_eq!(empty.kind, EventKind::Undo);
    assert_eq!(empty.erase_units, 2);
    assert!(empty.output.is_empty());
    assert_eq!(s.history_len(), 0);

    input(&mut s, "KA-");
    assert_eq!(s.process("-U").unwrap().erase_units, 1);
    assert_eq!(s.process("-U").unwrap().erase_units, 2);
    assert_eq!(s.process("-U").unwrap().erase_units, 2);

    input(&mut s, "n-*");
    input(&mut s, "-An"); // cursor operation clears the history
    assert_eq!(s.history_len(), 0);
    assert_eq!(s.process("-U").unwrap().erase_units, 2);
    assert_eq!(s.state().context, Context::Unknown);
    assert_eq!(s.state().last_vowel(), "A");

    let mut no_history = Session::<0>::new();
    assert_eq!(no_history.process("-U").unwrap().erase_units, 2);
}
#[test]
fn undo_restores_vowel_and_pending_tsu() {
    let mut s = Session::<20>::new();
    input(&mut s, "KI-");
    let before = s.state();
    input(&mut s, "KA-");
    input(&mut s, "-U");
    assert_eq!(s.state(), before);
    s.set_typing_mode(Mode::Romaji).unwrap();
    input(&mut s, "#-n");
    assert_eq!(input(&mut s, "KAtk-").output.text.as_str(), "ka");
    assert!(s.state().pending_sokuon());
    assert_eq!(input(&mut s, "KA-").output.text.as_str(), "kka");
    input(&mut s, "-U");
    assert!(s.state().pending_sokuon());
    assert_eq!(input(&mut s, "TA-").output.text.as_str(), "tta");
}
#[test]
fn undo_does_not_revert_user_mode_selection() {
    let mut s = Session::<20>::new();
    input(&mut s, "KA-");
    input(&mut s, "#-n");
    input(&mut s, "-U");
    assert_eq!(s.state().mode, Mode::Romaji);
}
#[test]
fn repeat_uses_resolved_output() {
    let mut s = Session::<20>::new();
    input(&mut s, "n-*");
    input(&mut s, "k-k");
    for _ in 0..3 {
        assert_eq!(input(&mut s, "#-").output.text.as_str(), "のが");
    }
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "な");
    input(&mut s, "-U");
    assert_eq!(input(&mut s, "#-").output.text.as_str(), "のが");
}
#[test]
fn ring_history_wraps_and_zero_depth_is_valid() {
    let mut s = Session::<2>::new();
    input(&mut s, "KA-");
    input(&mut s, "KI-");
    let expected = s.state();
    input(&mut s, "KU-");
    assert_eq!(s.history_len(), 2);
    input(&mut s, "-U");
    assert_eq!(s.state(), expected);
    input(&mut s, "-U");
    let fallback = s.process("-U").unwrap();
    assert_eq!(fallback.kind, EventKind::Undo);
    assert_eq!(fallback.erase_units, 2);
    let mut s = Session::<0>::new();
    assert_eq!(input(&mut s, "KA-").output.text.as_str(), "か");
    assert_eq!(s.history_len(), 0);
}
#[test]
fn macro_record_replay_and_recording_order() {
    let mut s = Session::<20>::new();
    input(&mut s, "#n*");
    input(&mut s, "n-*");
    input(&mut s, "k-k");
    input(&mut s, "#-*");
    assert_eq!(s.recorded(0).unwrap().text.as_str(), "しているのが");
    assert_eq!(input(&mut s, "#n").output.text.as_str(), "しているのが");
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "な");
    input(&mut s, "#n*");
    input(&mut s, "#t*");
    input(&mut s, "KA-");
    input(&mut s, "#-*");
    assert_eq!(s.recording(0), Some(true));
    assert_eq!(s.recording(1), Some(false));
    input(&mut s, "KI-");
    assert_eq!(s.recorded(0).unwrap().text.as_str(), "かき");
    assert_eq!(s.recorded(1).unwrap().text.as_str(), "か");
    input(&mut s, "#n-*");
    assert_eq!(s.recording(0), Some(false));
}
#[test]
fn macro_records_rendered_text_without_reconversion() {
    for (mode, expected) in [(Mode::Romaji, "shiteirunoga,"), (Mode::Jis, "dwe.kt@<")] {
        let mut s = Session::<20>::new();
        s.set_typing_mode(mode).unwrap();
        input(&mut s, "#-n");
        input(&mut s, "#n*");
        input(&mut s, "n-*");
        input(&mut s, "k-nk");
        input(&mut s, "#-*");
        assert_eq!(s.recorded(0).unwrap().text.as_str(), expected);
        let saved = *s.recorded(0).unwrap();
        assert_eq!(input(&mut s, "#n").output.text, saved.text);
    }
}
#[test]
fn recorder_overflow_is_atomic() {
    let mut s = Session::<20>::new();
    input(&mut s, "#n*");
    loop {
        let before = s.state();
        let saved = *s.recorded(0).unwrap();
        let history = s.history_len();
        match s.process("n-*") {
            Ok(e) => {
                s.acknowledge(e.ticket.unwrap(), 4).unwrap();
            }
            Err(Error::Capacity) => {
                assert_eq!(s.state(), before);
                assert_eq!(*s.recorded(0).unwrap(), saved);
                assert_eq!(s.history_len(), history);
                break;
            }
            other => panic!("{other:?}"),
        }
    }
}
#[test]
fn queries_are_pure_and_invalid_input_is_atomic() {
    let mut s = Session::<20>::new();
    input(&mut s, "n-*");
    let before = s.state();
    for _ in 0..10 {
        let t = translate("k-k", before, true).unwrap();
        assert_eq!(t.output.text.as_str(), "のが");
    }
    assert_eq!(s.state(), before);
    for raw in ["", "garbage", "KA-xyz", "KA-**", "か", "--", "KK-"] {
        assert_eq!(s.process(raw).unwrap_err(), Error::InvalidStroke, "{raw}");
        assert_eq!(s.state(), before);
    }
}
#[test]
fn cursor_barriers_prevent_stale_undo() {
    let mut s = Session::<20>::new();
    input(&mut s, "n-*");
    let e = input(&mut s, "-A");
    assert_eq!(e.output.operations()[0].key, Key::Left);
    assert_eq!(s.history_len(), 0);
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "な");
    input(&mut s, "n-*");
    s.reset();
    assert_eq!(input(&mut s, "ntk-").output.text.as_str(), "へ");
}
#[test]
fn key_operations_and_text_are_distinct() -> Result<()> {
    let t = translate("n-", State::default(), true).unwrap();
    assert!(t.output.text.as_str().is_empty());
    assert_eq!(t.output.operations()[0].key, Key::Space);
    let t = translate("-TYI", State::default(), true).unwrap();
    assert_eq!(t.output.text.as_str(), "（）");
    assert_eq!(
        t.output.operations()[0],
        Operation {
            byte_offset: 6,
            key: Key::Left
        }
    );
    let mut out = Output::new();
    out.push_text("を")?;
    let mut state = State::default();
    state.mode = Mode::Jis;
    render(&mut out, &mut state)?;
    assert_eq!(out.operations()[0].key, Key::Shift0);
    Ok(())
}

#[test]
fn sokuon_flushes_before_key_operations_and_can_be_undone() {
    let mut s = Session::<20>::new();
    input(&mut s, "#-n");
    input(&mut s, "KAtk-");
    let e = input(&mut s, "-n");
    assert_eq!(e.output.text.as_str(), "ltsu");
    assert_eq!(
        e.output.operations()[0],
        Operation {
            byte_offset: 4,
            key: Key::Return
        }
    );
    assert!(!s.state().pending_sokuon());
    input(&mut s, "-U");
    assert!(s.state().pending_sokuon());
}
#[test]
fn stn_without_vowel_emits_only_extra_sound() {
    for (raw, kana, romaji) in [
        ("STN-", "", ""),
        ("STNtk-", "っ", "ltsu"),
        ("STNntk-", "ー", "-"),
        ("KAntk-STNtk", "かーっ", "ka-ltsu"),
    ] {
        let t = preview(raw, State::default());
        assert_eq!(t.output.text.as_str(), kana, "{raw} in Kana");
        let mut state = State::default();
        state.mode = Mode::Romaji;
        let t = preview(raw, state);
        assert_eq!(t.output.text.as_str(), romaji, "{raw} in Romaji");
        assert!(!t.next.pending_sokuon(), "{raw} left a pending sokuon");
    }
    let mut state = State::default();
    state.mode = Mode::Jis;
    let t = preview("STNtk-", state);
    assert_eq!(t.output.text.as_str(), "Z");
    assert!(!t.next.pending_sokuon());
    // A regular trailing small-tsu still carries to the next stroke.
    let mut state = State::default();
    state.mode = Mode::Romaji;
    let t = preview("KAtk-", state);
    assert_eq!(t.output.text.as_str(), "ka");
    assert!(t.next.pending_sokuon());
}
#[test]
fn empty_stn_output_preserves_committed_context() {
    let mut state = preview("n-*", State::default()).next;
    state.mode = Mode::Romaji;
    let t = preview("STN-", state);
    assert_eq!(t.output.text.as_str(), "");
    assert!(!t.next.pending_sokuon());
    assert_eq!(t.next.context, Context::Nominalizable);
}
#[test]
fn multi_recorder_overflow_does_not_partially_record() {
    let mut s = Session::<20>::new();
    input(&mut s, "#n*");
    for _ in 0..42 {
        input(&mut s, "n-*");
    }
    input(&mut s, "#t*");
    let state = s.state();
    assert_eq!(s.process("n-*").unwrap_err(), Error::Capacity);
    assert_eq!(s.state(), state);
    assert!(s.recorded(1).unwrap().is_empty());
}
#[test]
fn c_interface_matches_rust_and_rejects_invalid_states() {
    use mejiro_engine::ffi::*;
    let raw = b"Kntk-Tk";
    let state = CState::default();
    let mut out = core::mem::MaybeUninit::<CTranslation>::uninit();
    let rc = unsafe { mejiro_core_translate(raw.as_ptr(), raw.len(), &state, 1, out.as_mut_ptr()) };
    assert_eq!(rc, 0);
    let out = unsafe { out.assume_init() };
    assert_eq!(&out.text[..out.text_len as usize], "ことへの".as_bytes());
    assert_eq!(core::mem::size_of::<CState>(), 4);
    assert_eq!(core::mem::size_of::<COperation>(), 4);
    assert_eq!(core::mem::size_of::<CTranslation>(), 556);
    let invalid = CState {
        vowel: 255,
        ..state
    };
    let rc = unsafe {
        mejiro_core_translate(raw.as_ptr(), raw.len(), &invalid, 1, core::ptr::null_mut())
    };
    assert_eq!(rc, -1);
    let mut sentinel = out;
    sentinel.text_len = 123;
    let rc = unsafe { mejiro_core_translate(raw.as_ptr(), raw.len(), &invalid, 1, &mut sentinel) };
    assert_eq!(rc, -1);
    assert_eq!(sentinel.text_len, 123);
    let raw = b"-U";
    assert_eq!(
        unsafe { mejiro_core_translate(raw.as_ptr(), raw.len(), &state, 1, &mut sentinel) },
        0
    );
    assert_eq!(sentinel.request, 1);
}

#[test]
fn overrides_break_context_and_undo_restores_it() {
    let mut s = Session::<20>::new();
    input(&mut s, "n-*");
    let before = s.state();
    let e = s.process_override("しいたけ").unwrap();
    s.acknowledge(e.ticket.unwrap(), 4).unwrap();
    assert_eq!(s.state().context, Context::Unknown);
    input(&mut s, "-U");
    assert_eq!(s.state(), before);
    let e = s.process_override("きれい").unwrap();
    s.acknowledge(e.ticket.unwrap(), 3).unwrap();
    assert_eq!(input(&mut s, "k-k").output.text.as_str(), "な");
}
#[test]
fn c_runtime_uses_common_history_without_allocator() {
    use mejiro_engine::ffi::*;
    let mut storage = core::mem::MaybeUninit::<Session<20>>::uninit();
    let ptr = storage.as_mut_ptr().cast::<u8>();
    assert_eq!(
        mejiro_core_session_size(),
        core::mem::size_of::<Session<20>>()
    );
    assert_eq!(
        unsafe { mejiro_core_session_init(ptr, mejiro_core_session_size()) },
        0
    );
    let mut event = core::mem::MaybeUninit::<CEvent>::uninit();
    let raw = b"KA-";
    assert_eq!(
        unsafe { mejiro_core_session_process(ptr, raw.as_ptr(), raw.len(), event.as_mut_ptr()) },
        0
    );
    let e = unsafe { event.assume_init() };
    assert_eq!(e.has_ticket, 1);
    assert_eq!(unsafe { mejiro_core_session_ack(ptr, e.ticket, 2) }, 0);
    let raw = b"-U";
    assert_eq!(
        unsafe { mejiro_core_session_process(ptr, raw.as_ptr(), raw.len(), event.as_mut_ptr()) },
        0
    );
    let e = unsafe { event.assume_init() };
    assert_eq!(e.kind, 1);
    assert_eq!(e.erase_units, 2);
    assert_eq!(
        unsafe { mejiro_core_session_process(ptr, raw.as_ptr(), raw.len(), event.as_mut_ptr()) },
        0
    );
    let fallback = unsafe { event.assume_init() };
    assert_eq!(fallback.kind, 1);
    assert_eq!(fallback.erase_units, 2);
    assert_eq!(fallback.has_ticket, 0);
}

#[test]
fn ime_off_follows_optional_system_selection() {
    for (stroke, keys) in [
        ("#-t", &[Key::ImeOn][..]),
        ("#-k", &[Key::SelectAlternateSystem, Key::ImeOff][..]),
    ] {
        let expected: Vec<_> = keys
            .iter()
            .map(|&key| Operation {
                byte_offset: 0,
                key,
            })
            .collect();
        let translation = translate(stroke, State::default(), true).unwrap();
        assert!(translation.output.text.as_str().is_empty());
        assert_eq!(translation.output.operations(), expected);
        let mut session = Session::<20>::new();
        assert_eq!(input(&mut session, stroke).output.operations(), expected);
    }
}

#[test]
fn rebuilt_verb_vocabulary_and_shortcuts() {
    for (stroke, expected) in [
        ("I-TNA*", "いただく"),
        ("I-TA*", "いたる"),
        ("KA-SNI*", "かんじる"),
        ("TNU-KU*", "つづく"),
        ("TU-TNU*", "つづける"),
        ("TA-TNA*", "ただす"),
        ("TA-NAU*", "たのしむ"),
        ("NI-SU*", "きにする"),
        ("NI-SUn*", "きにしない"),
        ("SAU-TIA*", "そだてる"),
        ("TKNIA-KYAU*", "べんきょうする"),
        ("SIA-*", "せつめいする"),
        ("KA-NI*", "かくにんする"),
        ("AU-SU*", "とおす"),
        ("IA-SU*", "かえす"),
        ("SKA-SU*", "かわす"),
        ("SNAU-SKNU*", "のぞむ"),
        ("KI-NA*", "きになる"),
        ("KI-NAn*", "きにならない"),
        ("KI-KNA*", "きがする"),
        ("KI-KNAn*", "きがしない"),
        ("SA-SIA*", "させる"),
        ("SA-STIA*", "される"),
        ("SA-TKNI*", "さびる"),
        ("TAU-*", "とする"),
        ("TAU-ST", "とる"),
        ("SAU-*", "そうする"),
        ("TNAU-*", "どうする"),
        ("A-ST", "ある"),
        ("A-STn", "ない"),
        ("KA-*", "かんがえる"),
    ] {
        assert_eq!(
            preview(stroke, State::default()).output.text.as_str(),
            expected,
            "{stroke}"
        );
    }
}
