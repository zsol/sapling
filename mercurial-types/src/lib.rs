// Copyright (c) 2004-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

#![deny(warnings)]
// TODO: (sid0) T21726029 tokio/futures deprecated a bunch of stuff, clean it all up
#![allow(deprecated)]

extern crate ascii;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate rust_crypto;
#[macro_use]
extern crate url;

extern crate futures;

#[macro_use]
extern crate error_chain;

#[cfg_attr(test, macro_use)]
extern crate quickcheck;

extern crate bookmarks;

extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod bdiff;
pub mod delta;
pub mod errors;
pub mod hash;
pub mod nodehash;
pub mod path;
pub mod utils;
pub mod repo;
pub mod manifest;
pub mod blob;
pub mod blobnode;
pub mod changeset;
mod node;

pub use blob::{Blob, BlobHash};
pub use blobnode::{BlobNode, Parents};
pub use changeset::{Changeset, Time};
pub use delta::Delta;
pub use manifest::{Entry, Manifest, Type};
pub use node::Node;
pub use nodehash::{NodeHash, NULL_HASH};
pub use path::Path;
pub use repo::{BoxRepo, Repo};
pub use utils::percent_encode;

pub use errors::{Error, ErrorKind};
