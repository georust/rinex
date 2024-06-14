##  GeoRust RINEX toolbox Issue Template

Thank you for using our toolbox and trying to make it better 🛰️ 🌍

When reporting applications issues, please follow the first paragraph.  
When reporting errors in the core libraries, refer to the second paragraph.

## Applications error reporting

- [ ] Make sure you are using the [latest revision](https://github.com/georust/rinex/releases) of the application
- [ ] Provide the faulty command line so we can reproduce the error
  - [ ] if the error is triggered by external data (not hosted within this repo), please be specific about the type of files and revisions being used
- [ ] Attach the application logs (use $RUST_LOG - environment variable) to your issue, either by direct Copy/Pasting
       or by attaching a separate file (you can use compression)

## Core Library error reporting

- [ ] Make sure you are using the [latest revision](https://github.com/georust/rinex/releases) of the library
- [ ] Provide a minimal reproducible example, so we can reproduce the error
