Constellation
=============

## SBAS

SBAS are geostationary augmentation systems,
usually to enhance the spatial solver performances.

### Selection helper

If you compile the crate with the `geo` feature,
you can access the SBAS selection helper method,
which helps select which SBAS augmentation to use,
depending on current location on Earth

```shell
cargo build --features with-geo
```

```rust
let paris = (48.808378, 2.382682); // lat, lon [ddeg]
let sbas = sbas_selection_helper(paris.0, paris.1);
assert_eq!(sbas, Some(Augmentation::EGNOS));

let antartica = (-77.490631,  91.435181); // lat, lon [ddeg]
let sbas = sbas_selection_helper(antartica.0, antartica.1);
assert_eq!(sbas.is_none(), true);
```
