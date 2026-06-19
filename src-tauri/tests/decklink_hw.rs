// Hardware-in-the-loop check: enumerate real DeckLink devices through the
// production FFI path.
//
// With `--features decklink` on a host that has Desktop Video installed this
// talks to the actual card; otherwise it exercises the mock path. The test only
// asserts that enumeration does not error — device counts are environment
// dependent — and prints what it finds (run with `-- --nocapture`).

#[test]
fn enumerate_decklink_hardware() {
    let version = caspar_server_gui_lib::decklink_api_version()
        .expect("DeckLink API version query should not error");
    println!("DeckLink API version: {:?}", version);

    let devices = caspar_server_gui_lib::enumerate_decklink_devices()
        .expect("DeckLink device enumeration should not error");

    println!("Enumerated {} DeckLink device(s):", devices.len());
    for d in &devices {
        println!(
            "  [{}] {} | id={} label={:?} in={:?} out={:?} \
             int_key={} ext_key={} duplex={} max_audio={}",
            d.index,
            d.model_name,
            d.persistent_id,
            d.device_label,
            d.input_connectors,
            d.output_connectors,
            d.supports_internal_keying,
            d.supports_external_keying,
            d.supports_duplex,
            d.max_audio_channels
        );
    }
}

// Determinism check for the COM-apartment fix: Tauri dispatches commands across
// a pool of worker threads, so enumeration must succeed on threads that never
// ran the one-shot global init. Each spawned thread must return the same count.
#[test]
fn enumerate_decklink_from_many_threads() {
    let baseline = caspar_server_gui_lib::enumerate_decklink_devices()
        .expect("baseline enumeration should not error")
        .len();

    let handles: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                let devices = caspar_server_gui_lib::enumerate_decklink_devices()
                    .unwrap_or_else(|e| panic!("thread {i} enumeration failed: {e}"));
                devices.len()
            })
        })
        .collect();

    for h in handles {
        let n = h.join().expect("worker thread panicked");
        assert_eq!(n, baseline, "device count must be stable across threads");
    }
    println!("All 8 worker threads enumerated {baseline} device(s) consistently");
}

// Live signal status via IDeckLinkStatus. With no input connected the card still
// reports a deterministic "not locked" — that is a valid, expected result.
#[test]
fn read_decklink_status() {
    let devices = caspar_server_gui_lib::enumerate_decklink_devices()
        .expect("enumeration should not error");
    if devices.is_empty() {
        println!("no DeckLink devices present — skipping status check");
        return;
    }

    let device = &devices[0];
    let status = caspar_server_gui_lib::decklink_device_status(device.index)
        .expect("status query should succeed for a present device");

    println!(
        "Device {} status: input_locked={} input={:?} ref_locked={} ref={:?} ref_type={:?}",
        device.index,
        status.input_signal_locked,
        status.input_display_mode,
        status.reference_signal_locked,
        status.reference_display_mode,
        status.reference_type
    );
}

// Non-destructive label write: read the current label, write a test value, read
// it back, then restore the original before asserting so the card is left as it
// was found even if the assertion fails.
#[test]
fn write_and_restore_decklink_label() {
    let devices = caspar_server_gui_lib::enumerate_decklink_devices()
        .expect("enumeration should not error");
    if devices.is_empty() {
        println!("no DeckLink devices present — skipping label write");
        return;
    }

    let id = devices[0].persistent_id.clone();
    let original = devices[0].device_label.clone().unwrap_or_default();

    caspar_server_gui_lib::set_decklink_device_label(&id, "TCIS-HW-TEST")
        .expect("label write should succeed");

    let after_write = caspar_server_gui_lib::enumerate_decklink_devices()
        .unwrap()
        .into_iter()
        .find(|d| d.persistent_id == id)
        .and_then(|d| d.device_label);

    // Restore first, so a failed assertion never leaves the card relabelled.
    let _ = caspar_server_gui_lib::set_decklink_device_label(&id, &original);

    assert_eq!(after_write.as_deref(), Some("TCIS-HW-TEST"));
    println!("label write verified and restored to {original:?}");
}
