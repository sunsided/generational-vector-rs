# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Added support for [smallvec](https://crates.io/crates/smallvec) via the `smallvec` feature.
- Added support for [tinyvec](https://crates.io/crates/tinyvec) via the `tinyvec` feature.

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
