#[cfg(test)]
mod tests {
    use crate::skills::{SkillPoints, SKILLS_SECTION_LENGTH};

    #[test]
    fn test_parse_and_write() {
        let bytes = [
            0x69, 0x66, 0x00, 0x01, 0x00, 0x14, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x14, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x14,
        ];

        let skill_points = SkillPoints::parse(&bytes).expect("skills should parse");
        assert_eq!(skill_points.points.len(), SKILLS_SECTION_LENGTH - 2);
        assert_eq!(skill_points.points[0], 0x00);
        assert_eq!(skill_points.points[3], 0x14);
        assert_eq!(skill_points.to_bytes(), bytes);
    }

    #[test]
    fn test_set_get() {
        let mut skill_points = SkillPoints::default();
        skill_points.set(5, 42);
        assert_eq!(skill_points.get(5), 42);
    }
}
