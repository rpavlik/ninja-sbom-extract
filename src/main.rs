// Copyright 2022-2023, Collabora, Ltd.
//
// SPDX-License-Identifier: BSL-1.0
//
// Author: Ryan Pavlik <ryan.pavlik@collabora.com>

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use petgraph::graphmap::DiGraphMap;
use spdx_rs::models::SPDX;

mod atom_table;
mod deps_file;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, derive_more::From, derive_more::Into,
)]
struct FileId(usize);

// #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
// enum File {
//     GeneratedFile(PathBuf),
//     SourceFile(PathBuf),
// }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    SourceFile,
    GeneratedFile,
    OutputArtifact,
}

impl FileType {
    fn promote_to(&mut self, mut new_type: Self) {
        if self < &mut new_type {
            *self = new_type
        }
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self::SourceFile
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
struct FileData {
    file_type: FileType,
}

struct SpdxGenerator {
    doc: SPDX,
    files: atom_table::AtomTable<PathBuf, FileId>,
    file_info: IndexMap<PathBuf, FileData>,
    graph: DiGraphMap<FileId, ()>,
}

impl SpdxGenerator {
    fn new(name: &str) -> Self {
        Self {
            doc: SPDX::new(name),
            files: Default::default(),
            file_info: Default::default(),
            graph: Default::default(),
        }
    }

    fn add_or_get_file_id(&mut self, path: PathBuf, file_type: FileType) -> FileId {
        match self.file_info.entry(path) {
            indexmap::map::Entry::Occupied(mut e) => {
                e.get_mut().file_type.promote_to(file_type);
                e.index().into()
            }
            indexmap::map::Entry::Vacant(e) => {
                let index = e.index();
                e.insert(Default::default()).file_type.promote_to(file_type);
                index.into()
            }
        }
    }
    fn add_dep(&mut self, product: PathBuf, input: PathBuf) {
        let product = self.add_or_get_file_id(product, FileType::GeneratedFile);
        let input = self.add_or_get_file_id(input, FileType::SourceFile);

        self.graph.add_edge(product, input, ());
    }
}

fn main() {
    let mut doc = SPDX::new("extracted");
    println!("Hello, world!");
}
