/// Tests that two properties can be packed together without affecting each other.
pub fn test_packed_properties<T, F1, F2, F3, F4>(
    instance: &mut T,
    setter1: F1,
    getter1: F2,
    setter2: F3,
    getter2: F4,
    value1: u64,
    value2: u64,
) where
    F1: FnOnce(&mut T, u64),
    F2: FnOnce(&T) -> u64,
    F3: FnOnce(&mut T, u64),
    F4: FnOnce(&T) -> u64,
{
    setter1(instance, value1);
    setter2(instance, value2);
    assert_eq!(getter1(instance), value1);
    assert_eq!(getter2(instance), value2);
}

/// Asserts that a property has the expected number of bits.
pub fn assert_size_bits<T, F1, F2>(instance: &mut T, setter: F1, getter: F2, expected_bits: u32)
where
    F1: Fn(&mut T, u64),
    F2: Fn(&T) -> u64,
{
    let max_value = (1u64 << expected_bits) - 1;
    setter(instance, max_value);
    assert_eq!(getter(instance), max_value);

    setter(instance, max_value + 1);
    assert_eq!(getter(instance), 0);
}
