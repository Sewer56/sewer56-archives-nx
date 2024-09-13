use hashbrown::HashMap;

use crate::api::traits::has_relative_path::HasRelativePath;

/// Groups the given files by extension.
///
/// Grouping the files by extension (similar to the 7zip `qs` parameter)
/// improves compression ratio, as data between different files of the same
/// type is likely to be similar.
///
/// For example, if you have two text files, two images, and two audio files,
/// provided their extensions match, they should be grouped together.
///
/// The Nx packing pipeline typically starts with the following steps:
/// - Sort files ascending by size [`sort_lexicographically`]
/// - Group files by extension (👈 This function ‼️)
///
/// [`sort_lexicographically`]: crate::utilities::arrange::sort_lexicographically
pub fn group_files<T>(files: Vec<T>) -> HashMap<String, Vec<T>>
where
    T: HasRelativePath,
{
    // Initialize the results HashMap with an estimated capacity.
    let capacity = (files.len() as f64).sqrt() as usize;
    let mut results: HashMap<String, Vec<T>> = HashMap::with_capacity(capacity);

    for file in files {
        // Extract the file extension from the relative path.
        let extension = extract_extension(file.relative_path()).to_string();

        // Insert the file into the appropriate group.
        results.entry(extension).or_default().push(file);
    }

    results
}

fn extract_extension(path: &str) -> &str {
    match path.rfind('.') {
        Some(dot_index) if dot_index > 0 && dot_index < path.len() - 1 => &path[dot_index + 1..],
        _ => "",
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::api::traits::has_relative_path::HasRelativePath;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SortTestItem {
        relative_path: String,
        size: u64,
    }

    impl SortTestItem {
        fn new(relative_path: &str, size: u64) -> Self {
            Self {
                relative_path: relative_path.to_string(),
                size,
            }
        }
    }

    impl HasRelativePath for SortTestItem {
        fn relative_path(&self) -> &str {
            &self.relative_path
        }
    }

    #[test]
    pub fn can_group_by_extension_preserving_size_ascending() {
        // Create the expected data
        let mut expected: HashMap<String, Vec<SortTestItem>> = HashMap::new();

        expected.insert(
            "txt".to_string(),
            vec![
                SortTestItem::new("fluffy.txt", 100),
                SortTestItem::new("whiskers.txt", 200),
                SortTestItem::new("mittens.txt", 300),
                SortTestItem::new("snickers.txt", 400),
                SortTestItem::new("tigger.txt", 500),
                SortTestItem::new("boots.txt", 600),
                SortTestItem::new("simba.txt", 700),
                SortTestItem::new("garfield.txt", 800),
                SortTestItem::new("nala.txt", 900),
                SortTestItem::new("cleo.txt", 1000),
            ],
        );

        expected.insert(
            "bin".to_string(),
            vec![
                SortTestItem::new("banana.bin", 450),
                SortTestItem::new("orange.bin", 666),
                SortTestItem::new("pear.bin", 777),
                SortTestItem::new("peach.bin", 888),
            ],
        );

        expected.insert(
            "pak".to_string(),
            vec![
                SortTestItem::new("data01.pak", 111),
                SortTestItem::new("data02.pak", 222),
                SortTestItem::new("data03.pak", 444),
                SortTestItem::new("data04.pak", 889),
            ],
        );

        // Flatten the expected values into a vector
        // This gives us a raw list of files in no particular size order.
        let mut items: Vec<SortTestItem> = expected.values().flat_map(|v| v.clone()).collect();

        // Sort the items by size ascending (replicate sort in packer)
        items.sort_by(|a, b| a.size.cmp(&b.size));

        // Now group the files using group_files function
        let groups = group_files(items.clone());

        // Now check that groups match expected
        // Each group should have its files sorted in order.
        for (ext, group_items) in groups {
            assert!(expected.contains_key(&ext));
            let expected_values = &expected[&ext];
            assert_eq!(expected_values, &group_items);
        }
    }
}
