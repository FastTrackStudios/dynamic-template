use daw_proto::{assert_tracks_equal, TrackGroup, TrackStructureBuilder};
use dynamic_template::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn snowmine_curfews() -> Result<()> {
    // -- Setup & Fixtures
    // Snowmine - Curfews: 11-stem indie pop session from MedleyDB. Auxiliary percussion,
    // electric bass, lead + backing vocals, vibraphone, drums, sampler, 2 clean electric
    // guitars, synthesizer, and tack piano. Tests eclectic indie instrumentation with
    // vibraphone and sampler classification.
    let items = vec![
        "01_AuxPerc.wav",
        "02_ElecBass.wav",
        "03_LeadVox.wav",
        "04_Vibraphone.wav",
        "05_Drums.wav",
        "06_Sampler.wav",
        "07_ElecGtr1.wav",
        "08_ElecGtr2.wav",
        "09_Synth.wav",
        "10_BackingVox.wav",
        "11_TackPiano.wav",
    ];
    let config = default_config();

    // -- Exec
    let tracks = items.organize_into_tracks(&config, None)?;

    // -- Check
    println!("\nTrack list:");
    daw_proto::display_tracklist(&tracks);

    // ============================================================================
    // Expected structure
    // ============================================================================

    let drums = TrackGroup::single_track("Drums", "05_Drums.wav");

    // Vibraphone dual-classified
    let percussion = TrackGroup::folder("Percussion")
        .track("Aux")
        .item("01_AuxPerc.wav")
        .track("Orch")
        .item("04_Vibraphone.wav")
        .end();

    // ElecBass classified as Electric guitar
    let guitars = TrackGroup::folder("Guitars")
        .track("Electric 1")
        .item("02_ElecBass.wav")
        .track("Electric 2")
        .item("07_ElecGtr1.wav")
        .track("Electric 3")
        .item("08_ElecGtr2.wav")
        .end();

    let keys = TrackGroup::single_track("Keys", "11_TackPiano.wav");
    let synths = TrackGroup::single_track("Synths", "09_Synth.wav");

    let vocals = TrackGroup::folder("Vocals")
        .track("Lead")
        .item("03_LeadVox.wav")
        .track("BGVs")
        .item("10_BackingVox.wav")
        .end();

    let orchestra = TrackGroup::single_track("Orchestra", "04_Vibraphone.wav");
    let unsorted = TrackGroup::single_track("Unsorted", "06_Sampler.wav");

    let expected = TrackStructureBuilder::new()
        .group(drums)
        .group(percussion)
        .group(guitars)
        .group(keys)
        .group(synths)
        .group(vocals)
        .group(orchestra)
        .group(unsorted)
        .build();

    assert_tracks_equal(&tracks, &expected)?;

    Ok(())
}
