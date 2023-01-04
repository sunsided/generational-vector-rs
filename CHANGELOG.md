# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Added support for [smallvec](https://crates.io/crates/smallvec) via the `smallvec` feature.
- Added support for [tinyvec](https://crates.io/crates/tinyvec) via the `tinyvec` feature.

## Changed

- Some frequently used functions were marked as `const` and `inline(always)`.
- The internal `len` property of the vector was removed and the length is now dynamically calculated
  from the data and free lists.
- The `remove` function now uses `Borrow<GenerationalIndex>` to feel a bit more natural.

## 0.2.1 - 2022-10-30

### Added

- `GenerationalVector` now implements `Debug`.

## 0.2.0 - 2022-04-15

### Added

- Added implementation of `From<Vec<T>>` trait.
- Added `GenerationalVector::<T>::new_from_iter()` to construct from
  arbitrary `IntoIterator<Item = T>` types.
- Added `IntoIter` implementations for `GenerationalVector<T>`, `&GenerationalVector<T>` and `&mut GenerationalVector<T>`.
- Added `iter()` and `iter_mut()` functions.

## 0.1.0 - 2022-04-15

### Added

- Initial release providing the `GenerationalVector<T>` type.
