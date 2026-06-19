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
