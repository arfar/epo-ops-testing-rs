# EPO OPS Scratch Space

This repo is mostly just about me trying to figure out what sort of information I can obtain from the [EPO OPS](https://www.epo.org/en/searching-for-patents/data/web-services/ops) service and how to do so.

This is very much a WIP and not actually something anyone can use easily. Currently, to obtain any patent information, you need to modify the contents of `main.rs`. There is not even a CLI interface yet.

## Requirements

* `rust`
* EPO OPS API keys - obtained by creating an account and an application with the EPO OPS. See the "Getting Started" page [here](https://www.epo.org/en/searching-for-patents/data/web-services/ops).

## Installation

* Download this repo (`git clone ...`)
* Copy and edit the configuration file:
  + `cp example_conf.ini conf.ini`
  + Edit to add your keys & destination file for the json files
* Build it `cargo build`

## Running

While crossing your fingers, try running `cargo run`. With any luck, it'll just work.

The error handling is very minimal and will likely just panic if it encounters and error. You can hopefully reason your way around the code relatively easily (it's not soooo many lines that you can't find where the error is). There are some debug messages if you wanted too. They can be turned on like so: `RUST_LOG=debug cargo run`.

## Features

* Authenticates to EPO OPS (a bare minimum feature).
* Looking up specific applications by application or publication number. All 4 constituents
* Searching the resigster with automatically obtaining all pages of results.
* Obtaining usage details.

### Unimplemented Features

* Anything in the family service.
* Bulk biblio information.
* Anything in the "published-data" services (i.e. images, abstracts, patent descriptions, etc).
* Number conversions.
* Anything in the "legal" section.

## Helpful Links

* 99% of the development has been done using the manual found [here](https://link.epo.org/web/searching-for-patents/data/en-ops-v3.2-documentation-version-1.3.20.pdf). It's good, but it's a bit of a beast.

## License

AGPLv3

(not that I would recommend anyone use this software at all currently)
