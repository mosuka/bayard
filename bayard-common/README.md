# Bayard

[![Join the chat at https://gitter.im/bayard-search/bayard](https://badges.gitter.im/bayard-search/bayard.svg)](https://gitter.im/bayard-search/bayard?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Bayard is a full-text search and indexing server written in [Rust](https://www.rust-lang.org/) built on top of [Tantivy](https://github.com/tantivy-search/tantivy) that implements [Raft Consensus Algorithm](https://raft.github.io/) and [gRPC](https://grpc.io/).  
Achieves consensus across all the nodes, ensures every change made to the system is made to a quorum of nodes.  
Bayard makes easy for programmers to develop search applications with advanced features and high availability.
