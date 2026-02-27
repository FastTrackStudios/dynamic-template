use daw_proto::{assert_tracks_equal, TrackGroup, TrackStructureBuilder};
use dynamic_template::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn karim_douaidy_hopscotch() -> Result<()> {
    // -- Setup & Fixtures
    // Karim Douaidy - Hopscotch: 9-stem Middle Eastern/world music session from MedleyDB.
    // Oud (Middle Eastern lute), darbuka and doumbek (goblet drums), acoustic guitar,
    // hand claps, auxiliary percussion, piano, and bass. Tests non-Western percussion
    // and instrument classification.
    let items = vec![
        "01_Oud.wav",
        "02_Darbuka.wav",
        "03_Doumbek.wav",
        "04_AcousticGtr.wav",
        "05_Claps.wav",
        "06_AuxPerc.wav",
        "07_Piano.wav",
        "08_Bass.wav",
        "09_LeadVox.wav",
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

    let percussion = TrackGroup::single_track("Percussion", "06_AuxPerc.wav");
    let bass = TrackGroup::single_track("Bass", "08_Bass.wav");
    let guitars = TrackGroup::single_track("Guitars", "04_AcousticGtr.wav");
    let keys = TrackGroup::single_track("Keys", "07_Piano.wav");
    let vocals = TrackGroup::single_track("Vocals", "09_LeadVox.wav");

    // Oud, darbuka, doumbek, claps → Unsorted (non-Western instruments)
    let unsorted = TrackGroup::folder("Unsorted")
        .track("Oud")
        .item("01_Oud.wav")
        .track("Darbuka")
        .item("02_Darbuka.wav")
        .track("Doumbek")
        .item("03_Doumbek.wav")
        .track("Claps")
        .item("05_Claps.wav")
        .end();

    let expected = TrackStructureBuilder::new()
        .group(percussion)
        .group(bass)
        .group(guitars)
        .group(keys)
        .group(vocals)
        .group(unsorted)
        .build();

    assert_tracks_equal(&tracks, &expected)?;

    Ok(())
}
