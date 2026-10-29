use mejiro_engine::{
    Context, Error, RuleKind, Session, State,
    ffi::{
        CRule, mejiro_core_session_rule_clear, mejiro_core_session_rule_count,
        mejiro_core_session_rule_get, mejiro_core_session_rule_hide, mejiro_core_session_rule_set,
    },
    translate,
};

fn text<const H: usize>(session: &Session<H>, stroke: &str) -> String {
    session
        .preview(stroke, true)
        .unwrap()
        .output
        .text
        .as_str()
        .into()
}

#[test]
fn user_rule_set_hide_restore_and_capacity_are_atomic() {
    let mut session = Session::<2>::new();
    let baseline = text(&session, "KA-");
    session
        .set_rule(RuleKind::User, "KA-", "犬", "", Context::Unknown)
        .unwrap();
    assert_eq!(text(&session, "KA-"), "犬");
    session
        .set_rule(RuleKind::User, "KA-", "猫", "", Context::Unknown)
        .unwrap();
    assert_eq!(text(&session, "KA-"), "猫");
    assert_eq!(session.rules().len(), 1);
    session.hide_rule(RuleKind::User, "KA-").unwrap();
    assert_eq!(text(&session, "KA-"), baseline);
    assert_eq!(session.rules().len(), 1);
    assert!(session.clear_rule(RuleKind::User, "KA-").unwrap());
    assert_eq!(session.rules().len(), 0);
    assert_eq!(
        session.set_rule(RuleKind::User, "-NA", "犬", "", Context::Unknown),
        Err(Error::InvalidStroke)
    );
    assert_eq!(text(&session, "KA-"), baseline);
    assert_eq!(
        session.set_rule(
            RuleKind::User,
            "KA-",
            &"あ".repeat(33),
            "",
            Context::Unknown
        ),
        Err(Error::Capacity)
    );
    assert_eq!(session.rules().len(), 0);
}

#[test]
fn general_and_verb_rules_use_grammar_and_inflection() {
    let mut session = Session::<2>::new();
    let baseline = text(&session, "K-T");
    session
        .set_rule(RuleKind::General, "K-T", "ここ", "", Context::Place)
        .unwrap();
    assert_eq!(text(&session, "K-T"), "ここ");
    let event = session.process("K-T").unwrap();
    session.acknowledge(event.ticket.unwrap(), 2).unwrap();
    assert_eq!(session.process("ntk-").unwrap().output.text.as_str(), "へ");
    session.hide_rule(RuleKind::General, "K-T").unwrap();
    assert_ne!(text(&session, "K-T"), baseline);
    session.clear_rule(RuleKind::General, "K-T").unwrap();
    assert_eq!(text(&session, "K-T"), baseline);

    let original = text(&session, "KA-KNA*");
    session
        .set_rule(RuleKind::Verb, "KA-KNA", "よ", "m", Context::Unknown)
        .unwrap();
    let edited = text(&session, "KA-KNA*");
    assert_ne!(edited, original);
    assert!(edited.starts_with("よ"));
    session.hide_rule(RuleKind::Verb, "KA-KNA").unwrap();
    assert_ne!(text(&session, "KA-KNA*"), original);
    session.clear_rule(RuleKind::Verb, "KA-KNA").unwrap();
    assert_eq!(text(&session, "KA-KNA*"), original);
    assert_eq!(
        session.set_rule(RuleKind::Verb, "KA-KNA", "よ", "invalid", Context::Unknown),
        Err(Error::InvalidState)
    );
    assert_eq!(
        translate("KA-KNA*", State::default(), true)
            .unwrap()
            .output
            .text
            .as_str(),
        original
    );
}

#[test]
fn composite_general_parts_can_be_edited() {
    let mut session = Session::<2>::new();
    session
        .set_rule(RuleKind::GeneralLeft, "K", "あの", "", Context::Unknown)
        .unwrap();
    session
        .set_rule(RuleKind::GeneralRight, "KAU", "場所", "", Context::Unknown)
        .unwrap();
    assert_eq!(text(&session, "K-KAU*"), "あの場所");
    session.hide_rule(RuleKind::GeneralRight, "KAU").unwrap();
    assert_ne!(text(&session, "K-KAU*"), "あの場所");
}

#[test]
fn c_api_round_trips_a_rule_and_rejects_invalid_utf8() {
    let mut session = Box::new(Session::<20>::new());
    let ptr = (&mut *session as *mut Session<20>).cast::<u8>();
    let mut rule = CRule {
        kind: RuleKind::User as u8,
        context: 0,
        hidden: 0,
        key_len: 3,
        value_len: 3,
        row_len: 0,
        reserved: [0; 2],
        key: [0; 32],
        value: [0; 96],
        row: [0; 8],
    };
    rule.key[..3].copy_from_slice(b"KA-");
    rule.value[..3].copy_from_slice("犬".as_bytes());
    assert_eq!(unsafe { mejiro_core_session_rule_set(ptr, &rule) }, 0);
    assert_eq!(unsafe { mejiro_core_session_rule_count(ptr) }, 1);
    let mut output = rule;
    assert_eq!(
        unsafe { mejiro_core_session_rule_get(ptr, 0, &mut output) },
        0
    );
    assert_eq!(&output.value[..3], "犬".as_bytes());
    assert_eq!(
        unsafe { mejiro_core_session_rule_get(ptr, 1, &mut output) },
        -3
    );
    rule.value[0] = 0xff;
    assert_eq!(unsafe { mejiro_core_session_rule_set(ptr, &rule) }, -1);
    assert_eq!(text(&session, "KA-"), "犬");
    assert_eq!(
        unsafe { mejiro_core_session_rule_hide(ptr, 0, b"KA-".as_ptr(), 3) },
        0
    );
    assert_eq!(text(&session, "KA-"), "か");
    assert_eq!(
        unsafe { mejiro_core_session_rule_clear(ptr, 0, b"KA-".as_ptr(), 3) },
        0
    );
    assert_eq!(unsafe { mejiro_core_session_rule_count(ptr) }, 0);
}
