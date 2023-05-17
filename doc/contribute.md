# Contribute 

Contributions are welcomed.

## New Navigation RINEX (NAV) revisions

New NAV revisions will require to update _navigation.json_ accordingly.

## New RINEX file types

To add support of a new RINEX file type, follow src/observation.rs or src/navigation.rs
when creating your new module.   

+ Create a new module src/module.rs
+ Define a new `Record` type inside that module.   
The `Record` type should define that RINEX file body content efficiently.   
At the moment, `records` are indexed by `epochs`. If another object would suite better to index
a new RINEX format, it might conflict with existing parts of the project and must be dealt with carefuly.

+ Declare the new enum value in the `src/record.rs` record enum with a new unwrapping method
+ Customize `src/record.rs::is_new_epoch` so the parser knows how to identify the new type
+ provide a new method to parse a block of content (usually an epoch) insde the new module `src/module.rs::build_new_record`

## TODO + Work in Progress

The `TODO` list describes what should be done in a near future

## Documentation

Documentation is fundamental, providing more examples of use mainly would be a good idea
