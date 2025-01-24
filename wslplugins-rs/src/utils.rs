#[cfg(test)]
pub(crate) fn test_transparence<T, U>() {
    assert_eq!(align_of::<T>(), align_of::<U>());
    assert_eq!(size_of::<T>(), size_of::<U>());
}
