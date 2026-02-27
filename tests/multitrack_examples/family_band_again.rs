use daw_proto::{assert_tracks_equal, TrackGroup, TrackStructureBuilder};
use dynamic_template::*;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn family_band_again() -> Result<()> {
    // -- Setup & Fixtures
    // Family Band - Again: 11-stem country/Americana session from MedleyDB. Electric bass,
    // backing + lead vocals, drums, acoustic guitar, clean + distorted electric guitars,
    // 2 lap steel guitars, melodica, and auxiliary percussion. Tests lap steel guitar
    // and melodica classification.
    let items = vec![
        "01_ElecBass.wav",
        "02_BackingVox.wav",
        "03_Drums.wav",
        "04_AcousticGtr.wav",
        "05_ElecGtr.wav",
        "06_DistortedGtr.wav",
        "07_LapSteel1.wav",
        "08_LapSteel2.wav",
        "09_LeadVox.wav",
        "10_Melodica.wav",
        "11_AuxPerc.wav",
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

    let drums = TrackGroup::single_track("Drums", "03_Drums.wav");
    let percussion = TrackGroup::single_track("Percussion", "11_AuxPerc.wav");

    // ElecBass classified as Electric, plus clean + distorted guitars
    let electric = TrackGroup::folder("Electric")
        .track("Electric 1")
        .item("01_ElecBass.wav")
        .track("Electric 2")
        .item("05_ElecGtr.wav")
        .track("Electric 3")
        .item("06_DistortedGtr.wav")
        .end();

    let steel = TrackGroup::folder("Steel")
        .track("Steel 1")
        .item("07_LapSteel1.wav")
        .track("Steel 2")
        .item("08_LapSteel2.wav")
        .end();

    let guitars = TrackGroup::folder("Guitars")
        .group(electric)
        .track("Acoustic")
        .item("04_AcousticGtr.wav")
        .group(steel)
        .end();

    let vocals = TrackGroup::folder("Vocals")
        .track("Lead")
        .item("09_LeadVox.wav")
        .track("BGVs")
        .item("02_BackingVox.wav")
        .end();

    // Melodica → Unsorted
    let unsorted = TrackGroup::single_track("Unsorted", "10_Melodica.wav");

    let expected = TrackStructureBuilder::new()
        .group(drums)
        .group(percussion)
        .group(guitars)
        .group(vocals)
        .group(unsorted)
        .build();

    assert_tracks_equal(&tracks, &expected)?;

    Ok(())
}
