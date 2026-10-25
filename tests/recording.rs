use mejiro_engine::*;
fn hid(kind: u8, usage: u16) -> HidEvent {
    HidEvent {
        usage,
        page: 7,
        mods: 1,
        kind,
        reserved: 0,
    }
}
#[test]
fn external_batch_is_record_only_and_preserves_interleaving() {
    let mut s = Session::<20>::new();
    s.process("#n*").unwrap();
    s.process("KA-").unwrap();
    s.reset();
    assert_eq!(s.recording(0), Some(true));
    s.record_external(&[hid(1, 6), hid(2, 6)]).unwrap();
    s.process("KI-").unwrap();
    s.process("#-*").unwrap();
    let replay = s.process("#n").unwrap();
    assert_eq!(replay.output.text.as_str(), "かき");
    let events = s.replayed_external();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].byte_offset, "か".len() as u16);
    assert_eq!(events[0].operation_index, 0);
    assert_eq!(events[0].event, hid(1, 6));
    assert!(replay.ticket.is_none()); // Keyboard edits are not guessable undo units.
}
#[test]
fn replay_records_into_other_slots_without_self_recursion() {
    let mut s = Session::<0>::new();
    s.process("#n*").unwrap();
    s.process("KA-").unwrap();
    s.record_external(&[hid(1, 6), hid(2, 6)]).unwrap();
    s.process("#t*").unwrap();
    s.process("#n").unwrap();
    assert_eq!(s.recorded(0).unwrap().text.as_str(), "か");
    assert_eq!(s.recorded_external(0).unwrap().len(), 2);
    assert_eq!(s.recorded(1).unwrap().text.as_str(), "か");
    assert_eq!(s.recorded_external(1).unwrap().len(), 2);
}
#[test]
fn capacity_and_invalid_batches_are_atomic_across_recorders() {
    let mut s = Session::<0>::new();
    s.process("#n*").unwrap();
    s.record_external(&[hid(1, 6); EXTERNAL_CAPACITY]).unwrap();
    s.process("#t*").unwrap();
    assert_eq!(s.record_external(&[hid(2, 6)]), Err(Error::Capacity));
    assert!(s.recorded_external(1).unwrap().is_empty());
    assert_eq!(s.record_external(&[hid(3, 6)]), Err(Error::InvalidState));
    s.abort_recordings();
    assert_eq!(s.recording(0), Some(false));
    assert!(s.recorded_external(0).unwrap().is_empty());
}
#[test]
fn semantic_operations_and_external_events_share_a_stable_anchor() {
    let mut s = Session::<0>::new();
    s.process("#n*").unwrap();
    s.process("-n").unwrap();
    s.record_external(&[hid(1, 6), hid(2, 6)]).unwrap();
    s.process("-n").unwrap();
    s.process("#-*").unwrap();
    let e = s.process("#n").unwrap();
    assert_eq!(e.output.operations().len(), 2);
    assert_eq!(s.replayed_external()[0].operation_index, 1);
}

#[test]
fn c_api_import_and_replay_query_have_no_partial_writes() {
    use mejiro_engine::ffi::*;
    let mut session = Session::<20>::new();
    session.process("#n*").unwrap();
    let session_ptr = (&mut session as *mut Session<20>).cast::<u8>();
    let events = [hid(1, 6), hid(2, 6)];
    unsafe {
        assert_eq!(
            mejiro_core_session_record_external(session_ptr, events.as_ptr(), events.len()),
            0
        );
        assert_eq!(
            mejiro_core_session_record_external(session_ptr, core::ptr::null(), 1),
            -1
        );
    }
    session.process("#-*").unwrap();
    session.process("#n").unwrap();
    let mut out = [RecordedEvent::default(); 2];
    unsafe {
        assert_eq!(
            mejiro_core_session_replayed_external(session_ptr, core::ptr::null_mut(), 0),
            2
        );
        assert_eq!(
            mejiro_core_session_replayed_external(session_ptr, out.as_mut_ptr(), 1),
            -2
        );
        assert_eq!(out[0], RecordedEvent::default());
        assert_eq!(
            mejiro_core_session_replayed_external(session_ptr, out.as_mut_ptr(), 2),
            2
        );
    }
    assert_eq!(out[0].event, events[0]);
}

#[test]
fn override_clears_previous_replay_events() {
    let mut s = Session::<0>::new();
    s.process("#n*").unwrap();
    s.record_external(&[hid(1, 6), hid(2, 6)]).unwrap();
    s.process("#-*").unwrap();
    s.process("#n").unwrap();
    assert_eq!(s.replayed_external().len(), 2);
    s.process_override("x").unwrap();
    assert!(s.replayed_external().is_empty());
}
