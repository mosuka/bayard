# Release notes
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## 0.9.0 (2022-08-03)
- Change architecture (#147) @mosuka

## 0.8.7 (2021-02-22)
- Bump up version to 0.8.7 #128 @mosuka
- Fix a bug that detect a wrong node ID #127 @mosuka

## 0.8.6 (2021-02-22)
- Bump up version to 0.8.6 #125 @mosuka
- Update dependencies #124 @mosuka
- Update docs #122 @mosuka
- added error messages #121 @barrotsteindev
- fix: REST API methods in documents #118 @hhatto

## 0.8.5 (2020-11-09)
- Bump up version to 0.8.5 #115 @mosuka
- Update workflows #114  @mosuka

## 0.8.4 (2020-11-09)
- Bump up version to 0.8.4 #113 @mosuka
- Resolve hostname #112 @mosuka
- Resolve hostname #111 @mosuka

## 0.8.3 (2020-11-07)
- Bump up version to 0.8.3 #110 @mosuka
- Update dependencies #109 @mosuka
- Fixed a bug that did not return schema #108 @mosuka
- Update Dockerfile #107 @mosuka

## 0.8.2 (2020-08-31)
- Bump up version #106 @mosuka
- Upgrade protobuf #105 @mosuka
- Upgrade dependencies #104 @mosuka
- Upgrade tantivy #103 @mosuka
- Fix docker run example #102 @mosuka
- Fix typo #101 @mosuka
- Migrate to Actix web #100 @mosuka

## 0.8.1 (2020-05-29)
- Bump up version #99 @mosuka
- Change web framework to Hyper #98 @mosuka
- Change the web framework of Metrics Server to Hyper #97 @mosuka
- Add example self-signed certificate #95 @mosuka
- Add GitHub Actions Integration #94 @messense
- Update jieba-rs to 0.5.0 #93 @messense
- Fix typo in CLI flag #92 @mosuka
- Support TLS on REST server #91 @mosuka
- Support CORS on REST server #90 @mosuka
- Merge CLI code #88 @mosuka
- Migrate to Actix web from Iron #87 @mosuka
- Refactor metrics server #86 @mosuka
- Combine the handlers into one file #85 @mosuka
- Refactor REST server #84 @mosuka

## 0.8.0 (2020-05-08)
- Add cli package #83 @mosuka

## 0.7.3 (2020-05-04)
- Bump up version #82 @mosuka

## 0.7.1 (2020-05-04)
- Update Dockerfile #81 @mosuka
- Upgrade dependencies #79 @mosuka

## 0.7.0 (2020-04-27)
- Split a package #78 @mosuka
- Update search.md #75 @klausondrag
- Refactoring #74 @mosuka
- Migrate to bayard-client #73 @mosuka
- Migrate to bayard-proto #72 @mosuka

## 0.6.0 (2020-02-25)
- Add Lindera tokenizer #69 @mosuka

## 0.5.0 (2020-02-23)
- Make the text anlyzers configurable #68 @mosuka
- Fix Dockerfile #66 @mosuka
- chore: Add Cargo.lock #65 @kenoss
- chore: Use docker layer cache #64 @kenoss

## 0.4.0 (2020-01-21)
- Initialize TokenizerManager #61 @mosuka

## 0.3.0 (2020-01-04)
- Add indexer flag #58 @mosuka
- Add docs for job scheduler #57 @mosuka
- Add document for gateway #56 @mosuka
- Update dependencies #55 @mosuka
- Bulk update #54 @mosuka
- Update docs #53 @mosuka
- Upgrade Tantivy to 0.11.3 #52 @mosuka
- Error handling #51 @mosuka
- Update docs #50 @mosuka
- Update to tantivy 0.11.2 and avoid blocking on merge while the IndexWriter's lock is held #49 @fulmicoton
- Support faceted search #48 @mosuka
- Add document for designing schema #47 @mosuka
- Get the document count that match the query #45 @mosuka


## 0.2.0 (2019-12-01)
- Add http server #41 @mosuka
- Add job scheduler #40 @mosuka
- Add probe command #39 @mosuka
- Add merge command #38 @mosuka
- Add rollback command #37 @mosuka
- Protobuf refactoring #33 @mosuka
- Bump up Tantivy #32 @mosuka
- Add metrics command #31 @mosuka
- Add command to commit index #29 @mosuka
- Make index writer to field of struct #28 @mosuka
- Add command to get schema #27 @mosuka
- Delete --leader-id from CLI #26 @mosuka
- Set default leader id #25 @mosuka
- Rename status to peers #24 @mosuka
- Add cluster status command #23 @mosuka
- Add signal handling #22 @mosuka
- Restore leave command document #21 @mosuka
- Add source code link #20 @mosuka
- Add documents #19 @mosuka
- Update Dockerfile #10 @mosuka
- Fixed typo in error message #6 @eko
- Dockerfile for tests and profit #5 @iyesin
- Small typo in get command example fixed #4 @msmakhlouf
- refactor get servers from ArgMatches #3 @robatipoor


## 0.1.0 (2019-11-01)
- First release by @mosuka
