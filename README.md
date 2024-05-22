# paradis

[![paradis crate](https://img.shields.io/crates/v/paradis.svg)](https://crates.io/crates/paradis)
[![paradis documentation](https://docs.rs/paradis/badge.svg)](https://docs.rs/paradis)

**`paradis` is currently at an early, experimental stage.
  Test coverage is deliberately poor in order to make it easier to iterate on the
  overall design. Community feedback is very welcome!**

`paradis` makes it easier to implement non-trivial parallel algorithms that require
access to a subset of indices into data structures that are structurally similar
to multidimensional arrays. It does so by providing abstractions at incrementally higher levels:

1. A low-level, unsafe abstraction for unsynchronized access to independent
   *records* of a collection.
2. Higher-level abstractions built on top of the unsafe base layer that allow many
   parallel access patterns to be expressed in safe code, or with a minimum of unsafe code.

The low-level abstractions are provided by the very lightweight `paradis-core` crate.
Library authors are encouraged to depend only on this crate in order to expose their
data structures for parallel access.

Please check out the [documentation](https://docs.rs/paradis) for more information
about how to use `paradis`.

## Examples

The examples given here are provided just to give you a taste of
the API. Please refer to the documentation for more context.

#### Safe parallel iteration with index lists

The following example shows how `paradis` can be used to safely iterate
over mutable elements located at arbitrary indices in a slice, in parallel. 

```rust
use paradis::index::{IndexList, narrow_access_to_indices};
use paradis::rayon::create_par_iter;
use rayon::iter::ParallelIterator;

let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
let indices = vec![4, 7, 1].check_unique().expect("Indices are unique");
let access = narrow_access_to_indices(data.as_mut_slice(), &indices)
    .expect("Indices are in bounds of the data structure");
create_par_iter(access).for_each(|x_i| *x_i = 0);

assert_eq!(data, vec![0, 0, 2, 3, 0, 5, 6, 0, 8, 9]);
```

#### Structured index lists

For some problems, the indices are *structured*. In this case, we may be able to
avoid runtime checks for uniqueness, and instead prove uniqueness by structured
construction, using *index combinators*. The example below shows how structured
uniqueness allows us to mutate the superdiagonal of a matrix.

```rust
use nalgebra::dmatrix;
use paradis::index::{IndexList, narrow_access_to_indices};
use paradis::rayon::create_par_iter;
use rayon::iter::ParallelIterator;

// Access implementation omitted
use paradis_demo::DMatrixParAccessMut;

let mut matrix = dmatrix![1, 1, 1, 1, 1;
                          1, 1, 1, 1, 1;
                          1, 1, 1, 1, 1];

// Superdiagonal indices are [(0, 1), (1, 2), (2, 3)]
let superdiagonal_indices = (0 .. 3).index_zip(1 .. 4);
let access = DMatrixParAccessMut::from_matrix_mut(&mut matrix);
let superdiagonal_access = narrow_access_to_indices(access, &superdiagonal_indices)
    .expect("Indices are in bounds");

create_par_iter(superdiagonal_access).for_each(|x_ij| *x_ij = 0);

assert_eq!(matrix,
           dmatrix![1, 0, 1, 1, 1;
                    1, 1, 0, 1, 1;
                    1, 1, 1, 0, 1]);
```

#### Low-level unsafe parallel access

The higher-level features of `paradis` are built on top of its low-level abstractions for *parallel access*.
The example below shows how we may use careful unsynchronized access to
mutate even and odd parts of a slice in different threads.

```rust
use paradis_core::{BoundedParAccess, IntoParAccess};
use std::thread::scope;

let mut data = vec![0; 100];
let n = data.len();
let access = data.into_par_access();

scope(|s| {
    s.spawn(|| {
        // The first thread touches elements at even indices
        for i in (0 .. n).step_by(2) {
            unsafe { *access.get_unsync(i) = 1; }
        }
    });

    s.spawn(|| {
        // The second thread touches elements at odd indices
        for i in (1 .. n).step_by(2) {
            unsafe { *access.get_unsync(i) = 2; }
        }
    });
})
```

## Contributing

`paradis` is open source, and contribution is welcome. There are several ways you can contribute:

1. by trying `paradis` out for your application and report 
   your experience in the [forum](https://github.com/Andlon/paradis/discussions).
2. by filing [issues](https://github.com/Andlon/paradis/issues), like bugs, ideas for improvement or concerns about the design.
3. by fixing bugs, improving documentation, or contributing new features as part of a pull request.
   Please note that not at all new features will necessarily be accepted.
   Before investing a great deal of time on new functionality, please file an issue
   to see if the feature is likely to be accepted.

Keep in mind that `paradis` is not developed professionally.
Although I, @Andlon, have every intention of following up on issues and PRs, life has a tendency
to get in the way at times, sometimes for extended periods of time.

## License

`paradis` is distributed under the terms of either the MIT license or the Apache License (Version 2.0), at your option.
See `LICENSE-APACHE` and `LICENSE-MIT` for details.

By contributing intellectual property to this repository, you agree to license your contribution
under the same terms.




   