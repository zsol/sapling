/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use blobstore::BlobstoreBytes;
use bytes::Bytes;
use fbthrift::compact_protocol;
use quickcheck::Arbitrary;
use quickcheck::Gen;

use crate::blob::Blob;
use crate::blob::BlobstoreValue;
use crate::blob::ContentMetadataBlob;
use crate::errors::ErrorKind;
use crate::hash;
use crate::thrift;
use crate::thrift_field;
use crate::typed_hash::ContentId;
use crate::typed_hash::ContentMetadataId;
use crate::ContentMetadataV2;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentAlias(ContentId);

impl ContentAlias {
    pub fn from_content_id(id: ContentId) -> Self {
        ContentAlias(id)
    }

    pub fn from_bytes(blob: Bytes) -> Result<Self> {
        let thrift_tc = compact_protocol::deserialize(blob.as_ref())
            .with_context(|| ErrorKind::BlobDeserializeError("ContentAlias".into()))?;
        Self::from_thrift(thrift_tc)
    }

    pub fn from_thrift(ca: thrift::ContentAlias) -> Result<Self> {
        match ca {
            thrift::ContentAlias::ContentId(id) => {
                Ok(Self::from_content_id(ContentId::from_thrift(id)?))
            }
            thrift::ContentAlias::UnknownField(x) => bail!(ErrorKind::InvalidThrift(
                "ContentAlias".into(),
                format!("unknown content alias field: {}", x)
            )),
        }
    }

    pub fn into_blob(self) -> BlobstoreBytes {
        let alias = thrift::ContentAlias::ContentId(self.0.into_thrift());
        let data = compact_protocol::serialize(&alias);
        BlobstoreBytes::from_bytes(data)
    }

    pub fn content_id(&self) -> ContentId {
        self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContentMetadata {
    pub total_size: u64,
    pub content_id: ContentId,
    pub sha1: hash::Sha1,
    pub sha256: hash::Sha256,
    pub git_sha1: hash::RichGitSha1,
}

impl ContentMetadata {
    pub fn from_thrift(cab: thrift::ContentMetadata) -> Result<Self> {
        let total_size = thrift_field!(ContentMetadata, cab, total_size)?;
        let total_size: u64 = total_size.try_into()?;

        let res = ContentMetadata {
            total_size,
            content_id: ContentId::from_thrift(thrift_field!(ContentMetadata, cab, content_id)?)?,
            sha1: hash::Sha1::from_bytes(&thrift_field!(ContentMetadata, cab, sha1)?.0)?,
            sha256: hash::Sha256::from_bytes(&thrift_field!(ContentMetadata, cab, sha256)?.0)?,
            git_sha1: hash::RichGitSha1::from_bytes(
                &thrift_field!(ContentMetadata, cab, git_sha1)?.0,
                "blob",
                total_size,
            )?,
        };

        Ok(res)
    }

    fn into_thrift(self) -> thrift::ContentMetadata {
        thrift::ContentMetadata {
            total_size: Some(self.total_size as i64),
            content_id: Some(self.content_id.into_thrift()),
            sha1: Some(self.sha1.into_thrift()),
            git_sha1: Some(self.git_sha1.into_thrift()),
            sha256: Some(self.sha256.into_thrift()),
        }
    }
}

impl std::convert::From<ContentMetadataV2> for ContentMetadata {
    fn from(data: ContentMetadataV2) -> Self {
        Self {
            content_id: data.content_id,
            git_sha1: data.git_sha1,
            sha1: data.sha1,
            sha256: data.sha256,
            total_size: data.total_size,
        }
    }
}

impl Arbitrary for ContentMetadata {
    fn arbitrary(g: &mut Gen) -> Self {
        // Large u64 values can't be represented in thrift
        let total_size = u64::arbitrary(g) / 2;

        Self {
            total_size,
            content_id: ContentId::arbitrary(g),
            sha1: hash::Sha1::arbitrary(g),
            sha256: hash::Sha256::arbitrary(g),
            git_sha1: hash::RichGitSha1::from_sha1(hash::GitSha1::arbitrary(g), "blob", total_size),
        }
    }
}

impl BlobstoreValue for ContentMetadata {
    type Key = ContentMetadataId;

    fn into_blob(self) -> ContentMetadataBlob {
        let id = From::from(self.content_id.clone());
        let thrift = self.into_thrift();
        let data = compact_protocol::serialize(&thrift);
        Blob::new(id, data)
    }

    fn from_blob(blob: ContentMetadataBlob) -> Result<Self> {
        let thrift_tc = compact_protocol::deserialize(blob.data().as_ref())
            .with_context(|| ErrorKind::BlobDeserializeError("ContentMetadata".into()))?;
        Self::from_thrift(thrift_tc)
    }
}

#[cfg(test)]
mod test {
    use quickcheck::quickcheck;

    use super::*;

    quickcheck! {
        fn content_metadata_thrift_roundtrip(cab: ContentMetadata) -> bool {
            let thrift_cab = cab.clone().into_thrift();
            let cab2 = ContentMetadata::from_thrift(thrift_cab)
                .expect("thrift roundtrips should always be valid");
            println!("cab: {:?}", cab);
            println!("cab2: {:?}", cab2);
            cab == cab2
        }

        fn content_metadata_blob_roundtrip(cab: ContentMetadata) -> bool {
            let blob = cab.clone().into_blob();
            let cab2 = ContentMetadata::from_blob(blob)
                .expect("blob roundtrips should always be valid");
            cab == cab2
        }
    }
}
