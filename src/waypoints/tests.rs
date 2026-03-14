use crate::{waypoints::*, Act};

const NONTRIVIAL_WAYPOINT_BYTES: [u8; 80] = [
    0x57, 0x53, 0x01, 0x00, 0x00, 0x00, 0x50, 0x00, 0x02, 0x01, 0xEF, 0xEB, 0xD7, 0xFF, 0x47, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x01, 0xEF, 0xE3, 0xBD, 0xFF, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0xEF, 0xEF, 0xEF, 0xFE, 0x43, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[test]
fn parse_waypoints_reads_expected_flags() {
    let parsed_waypoints = Waypoints::parse(&NONTRIVIAL_WAYPOINT_BYTES)
        .expect("Waypoint payload should parse in test fixture.");

    assert_eq!(parsed_waypoints.hell.act1.get_by_index(8), Ok(true));
    assert_eq!(parsed_waypoints.nightmare.act2.get_by_index(3), Ok(false));
}

#[test]
fn waypoints_round_trip_preserves_nontrivial_bytes() {
    let parsed_waypoints = Waypoints::parse(&NONTRIVIAL_WAYPOINT_BYTES)
        .expect("Waypoint payload should parse in test fixture.");
    assert_eq!(parsed_waypoints.to_bytes(), NONTRIVIAL_WAYPOINT_BYTES);
}

#[test]
fn default_waypoints_encode_to_expected_layout() {
    let mut expected_bytes = [0x00; 80];
    expected_bytes[0..8].copy_from_slice(&[0x57, 0x53, 0x01, 0x00, 0x00, 0x00, 0x50, 0x00]);
    for difficulty_start in [8, 32, 56] {
        expected_bytes[difficulty_start] = 0x02;
        expected_bytes[difficulty_start + 1] = 0x01;
        expected_bytes[difficulty_start + 2] = 0x01;
    }

    assert_eq!(expected_bytes, Waypoints::default().to_bytes());
}

#[test]
fn act_set_rejects_wrong_act_waypoint() {
    let mut act1 = ActWaypoints::<9>::new_for_act(Act::Act1);
    let error = act1
        .set(Waypoint::LutGholein, true)
        .expect_err("Setting an Act II waypoint on Act I must fail.");
    assert_eq!(
        error,
        WaypointError::WrongAct {
            waypoint: Waypoint::LutGholein,
            expected: Act::Act1,
            actual: Act::Act2,
        }
    );
}

#[test]
fn act_get_rejects_wrong_act_waypoint() {
    let act4 = ActWaypoints::<3>::new_for_act(Act::Act4);
    let error = act4
        .get(Waypoint::Harrogath)
        .expect_err("Reading an Act V waypoint from Act IV must fail.");
    assert_eq!(
        error,
        WaypointError::WrongAct {
            waypoint: Waypoint::Harrogath,
            expected: Act::Act4,
            actual: Act::Act5,
        }
    );
}

#[test]
fn get_by_index_reports_out_of_range() {
    let act4 = ActWaypoints::<3>::new_for_act(Act::Act4);
    let error = act4.get_by_index(3).expect_err("Act IV has indices 0..=2 only.");
    assert_eq!(
        error,
        WaypointError::IndexOutOfRange {
            act: Act::Act4,
            index: 3,
            max_index: 2,
        }
    );
}

#[test]
fn set_by_index_reports_out_of_range() {
    let mut act4 = ActWaypoints::<3>::new_for_act(Act::Act4);
    let error = act4.set_by_index(3, true).expect_err("Act IV has indices 0..=2 only.");
    assert_eq!(
        error,
        WaypointError::IndexOutOfRange {
            act: Act::Act4,
            index: 3,
            max_index: 2,
        }
    );
}

#[test]
fn difficulty_set_all_updates_every_act() {
    let mut difficulty_waypoints = DifficultyWaypoints::default();
    assert_eq!(difficulty_waypoints.act1.get_by_index(0), Ok(true));

    difficulty_waypoints.set_all(false);

    assert_eq!(difficulty_waypoints.act1.get_by_index(0), Ok(false));
    assert_eq!(difficulty_waypoints.act2.get_by_index(0), Ok(false));
    assert_eq!(difficulty_waypoints.act3.get_by_index(4), Ok(false));
    assert_eq!(difficulty_waypoints.act4.get_by_index(2), Ok(false));
    assert_eq!(difficulty_waypoints.act5.get_by_index(8), Ok(false));
}

#[test]
fn waypoint_try_from_rejects_invalid_absolute_index() {
    let _error = Waypoint::try_from(39).expect_err("Only ids 0..=38 are valid.");
}

#[test]
fn waypoint_from_act_index_rejects_invalid_index_for_act() {
    let _error =
        Waypoint::from_act_index(Act::Act4, 3).expect_err("Act IV only has indices 0..=2.");
}

#[test]
fn waypoint_metadata_methods_match_known_values() {
    let catacombs = Waypoint::Catacombs;
    assert_eq!(catacombs.act(), Act::Act1);
    assert_eq!(catacombs.name(), "Catacombs");
    assert_eq!(catacombs.index_within_act(), 8);

    let river_of_flames = Waypoint::RiverOfFlames;
    assert_eq!(river_of_flames.act(), Act::Act4);
    assert_eq!(river_of_flames.name(), "River of Flames");
    assert_eq!(river_of_flames.index_within_act(), 2);
}
