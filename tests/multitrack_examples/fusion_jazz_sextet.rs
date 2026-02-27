use daw_proto::{assert_tracks_equal, TrackGroup, TrackStructureBuilder};
use dynamic_template::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn fusion_jazz_sextet() -> Result<()> {
    // -- Setup & Fixtures
    // MusicDelta Fusion Jazz: 6-stem jazz fusion session from MedleyDB. Drum set,
    // electric bass, electric piano (Rhodes-style), synthesizer, soprano saxophone,
    // and trumpet. Tests jazz fusion with electric instruments and soprano sax.
    let items = vec![
        "01_Drums.wav",
        "02_ElecBass.wav",
        "03_ElecPiano.wav",
        "04_Synthesizer.wav",
        "05_SopranoSax.wav",
        "06_Trumpet.wav",
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

    // ElecBass and ElecPiano both classified as Electric guitars — naming ambiguity
    let guitars = TrackGroup::folder("Guitars")
        .track("Electric 1")
        .item("02_ElecBass.wav")
        .track("Electric 2")
        .item("03_ElecPiano.wav")
        .end();

    let synths = TrackGroup::single_track("Synths", "04_Synthesizer.wav");
    let horns = TrackGroup::single_track("Horns", "05_SopranoSax.wav");
    let orchestra = TrackGroup::single_track("Orchestra", "06_Trumpet.wav");

    let expected = TrackStructureBuilder::new()
        .group(drums)
        .group(guitars)
        .group(synths)
        .group(horns)
        .group(orchestra)
        .build();

    assert_tracks_equal(&tracks, &expected)?;

    Ok(())
}
