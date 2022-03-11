# Contribute 

Contributions are welcomed.

## New Navigation RINEX (NAV) revisions

New NAV revisions will require to update _navigation.json_ accordingly.

## New RINEX file types

To add support of a new RINEX file type, follow src/observation.rs or src/navigation.rs
when creating your new module.   

Define a new `Record` type definition inside that new module.   
The `Record` type should define that RINEX file body content efficiently.   

Add a new type of RINEX Record in src/record.rs and a new method to extract a block
section of interest (generally an _epoch_) when parsing the body, in src/record.rs:block\_record\_start.
