use daw_proto::{assert_tracks_equal, TrackGroup, TrackStructureBuilder};
use dynamic_template::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn gospel_session() -> Result<()> {
    // -- Setup & Fixtures
    // MusicDelta Gospel: 6-stem gospel session from MedleyDB. Drum set, electric bass,
    // 2 clean electric guitars, hand claps, and female lead vocal. Tests gospel
    // arrangement with claps classification.
    let items = vec![
        "01_Drums.wav",
        "02_ElecBass.wav",
        "03_ElecGtr1.wav",
        "04_ElecGtr2.wav",
        "05_Claps.wav",
        "06_LeadVox.wav",
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

    let drums = TrackGroup::single_track("Drums", "01_Drums.wav");

    // ElecBass classified as Electric guitar
    let guitars = TrackGroup::folder("Guitars")
        .track("Electric 1")
        .item("02_ElecBass.wav")
        .track("Electric 2")
        .item("03_ElecGtr1.wav")
        .track("Electric 3")
        .item("04_ElecGtr2.wav")
        .end();

    let vocals = TrackGroup::single_track("Vocals", "06_LeadVox.wav");

    // Claps → Unsorted
    let unsorted = TrackGroup::single_track("Unsorted", "05_Claps.wav");

    let expected = TrackStructureBuilder::new()
        .group(drums)
        .group(guitars)
        .group(vocals)
        .group(unsorted)
        .build();

    assert_tracks_equal(&tracks, &expected)?;

    Ok(())
}
