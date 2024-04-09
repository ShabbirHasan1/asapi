// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

pub mod bson_type;
pub mod delete;
pub mod find;
pub mod insert;

use bson::{Bson, Document};
use std::borrow::Cow;

#[derive(Debug)]
pub struct DocumentEntry<'a> {
    key: Cow<'a, str>,
    bs: Bson,
}

pub fn extract_document<'a>(doc: &'a Document) -> Vec<DocumentEntry<'a>> {
    doc.iter()
        .map(move |entry| {
            let (k, bs) = entry;

            // (k.to_owned(), bs.to_owned(), serde_json::json!(doc))
            DocumentEntry {
                key: Cow::Borrowed(k),
                bs: bs.to_owned(),
            }
        })
        .collect::<Vec<DocumentEntry>>()
}
