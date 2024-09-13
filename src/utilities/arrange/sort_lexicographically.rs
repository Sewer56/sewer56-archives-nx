use crate::api::traits::has_relative_path::HasRelativePath;

/// Helper function to sort items lexicographically.
///
/// # Arguments
///
/// * `items` - The items to sort.
///
/// # Remarks
///
/// It is assumed that the items passed to [sort_lexicographically] are partially sorted.
/// This is because they are likely to come from
pub fn sort_lexicographically<T: HasRelativePath>(items: &mut [T]) {
    items.sort_by(|a, b| a.relative_path().cmp(b.relative_path()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::{string::String, vec::Vec};
    use itertools::Itertools;
    use rstest::rstest;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct StringWrapper(String);

    impl HasRelativePath for StringWrapper {
        fn relative_path(&self) -> &str {
            &self.0
        }
    }

    fn generate_test_data_lexicographic() -> Vec<Vec<StringWrapper>> {
        let expected_result = [
            "Data/Movie/Credits.sfd",
            "Data/Movie/EN/Opening.sfd",
            "Data/Sonk.bin",
            "Sonk.exe",
        ];

        expected_result
            .iter()
            .permutations(expected_result.len())
            .map(|perm| {
                perm.into_iter()
                    .map(|&s| StringWrapper(s.to_string()))
                    .collect()
            })
            .collect()
    }

    #[rstest]
    #[case::permutations(generate_test_data_lexicographic())]
    fn sorts_lexicographically_with_all_permutations(
        #[case] permutations: Vec<Vec<StringWrapper>>,
    ) {
        let expected = vec![
            StringWrapper("Data/Movie/Credits.sfd".to_string()),
            StringWrapper("Data/Movie/EN/Opening.sfd".to_string()),
            StringWrapper("Data/Sonk.bin".to_string()),
            StringWrapper("Sonk.exe".to_string()),
        ];

        for mut files in permutations {
            sort_lexicographically(&mut files);

            assert_eq!(
                files, expected,
                "Sorting failed for permutation: {:?}",
                files
            );
        }
    }
}
