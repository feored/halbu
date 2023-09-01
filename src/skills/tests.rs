#[cfg(test)]
mod tests {
    use crate::skills::*;

    #[test]
    fn test_parse_and_write() {
        let byte_vector = [
            0x69, 0x66, 0x00, 0x01, 0x00, 0x14, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x14, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x14,
        ];

        let skills = SkillSet::parse(&byte_vector, Class::Sorceress);
        // Teleport
        assert_eq!(
            skills.0[18],
            Skill {
                name: String::from("Teleport"),
                skilldesc: String::from("teleport"),
                id: 54,
                points: 1
            }
        );

        let result = skills.to_bytes();

        assert_eq!(result, byte_vector);
    }

    #[test]
    fn test_parse() {
        let byte_vector = [
            0x69, 0x66, 0x00, 0x01, 0x00, 0x14, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x14, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x14,
        ];

        let skills = SkillSet::parse(&byte_vector, Class::Sorceress);
        // Ice blast
        assert_eq!(
            skills.0[9],
            Skill {
                points: 17,
                name: String::from("Ice Blast"),
                id: 45,
                skilldesc: String::from("ice blast")
            }
        );
    }
}
