# RINEX/GNSS QC and processing

This crate is a small library to share and implement in other libraries
to form a coherent ecosystem to process and analyze GNSS data.

As an example, this crate is implemented in the RINEX and SP3 libraries, the RINEX-QC
library and the RINEX-Cli application and allows the synthesis of analysis reports
and the processing on GNSS down to navigation.

## Existing Modules

- html: HTML report rendition
- merge: describes how we stack data into an already existing context
- processing: available on crate feature only,
describes a filter designer and processing ops
