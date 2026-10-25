# Mixed host and steno recordings

`Session` owns recording controls and all seven saved slots. Plover can continue
using ordinary `process` outputs without supplying keyboard input. QMK/RMK can
buffer already-emitted HID events while typing and submit them with
`record_external` before the next steno input. Importing records data only; it
does not send or replay those events.

`reset` clears linguistic context and undo history, preserving recording state
and saved contents. Only an explicit recording control stops a normal recording.
`abort_recordings` stops and erases active slots after capture failure, leaving
unrelated saved slots intact.

Each recording consists of its existing `Output` plus `recorded_external(slot)`.
The latter contains HID usage page/usage, modifier bits and down/up/modifier
events. Anchors give the UTF-8 byte offset and number of semantic operations
already executed. Merge both streams in that order when replaying. After
`process` returns a replay event, `replayed_external()` exposes its external
stream. An external-only replay can have an empty ordinary Output. Such replays
are undo barriers: the core cannot infer what an external key did in the host.

Replaying does not append a slot to itself while it is being recorded, but does
record into other active slots. The host must suppress feedback from its own
replay transport. Saved HID usage/modifier events describe the resolved host
output; they are not reinterpreted through a new letter layout. A recording made
for a JIS host or Mac therefore retains those host-specific output strokes.
Timing, OS autorepeat and the host IME's final edited text are not captured.

Capacity is 512 UTF-8 bytes, eight semantic operations, and 128 external events
per slot. Batch import is atomic across all active recorders. On overflow it
returns `Error::Capacity` without partially importing the batch. A firmware must
keep live typing working and explicitly abort affected active recordings rather
than silently truncate them. It must release replay-owned keys and restore the
live modifier state at the end of playback, including failed playback.

The additive C API uses `mejiro_hid_event` and `mejiro_recorded_event` declared in
`native/mejiro.h`. `mejiro_core_session_record_external` imports a batch;
`mejiro_core_session_replayed_external` queries/copies the latest replay stream;
`mejiro_core_session_abort_recordings` handles failed capture. Existing
translation/event structs remain unchanged. Opaque Session storage must always
use the current library's reported size and alignment.
