// Copyright 2022-2023, Collabora, Ltd.
//
// SPDX-License-Identifier: BSL-1.0
//
// Author: Ryan Pavlik <ryan.pavlik@collabora.com>

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use deps_file::DepsForOneFile;
use indexmap::IndexMap;
use petgraph::graphmap::DiGraphMap;
use query_result::QueryResult;
use spdx_rs::models::SPDX;

use crate::deps_file::recognize_deps;

mod atom_table;
mod deps_file;
mod query_result;

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
    root: PathBuf,
    file_info: IndexMap<PathBuf, FileData>,
    graph: DiGraphMap<FileId, ()>,
}

impl SpdxGenerator {
    fn new(name: &str, root: PathBuf) -> Self {
        Self {
            doc: SPDX::new(name),
            root,
            file_info: Default::default(),
            graph: Default::default(),
        }
    }

    fn add_or_get_file_id(&mut self, path: PathBuf, file_type: FileType) -> FileId {
        let mut path = path;
        if path.is_relative() {}
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
    fn add_deps(&mut self, deps: DepsForOneFile<'_>) {
        let product = self.add_or_get_file_id(deps.output.to_owned(), FileType::GeneratedFile);
        for input in deps.inputs {
            let input = self.add_or_get_file_id(input.to_owned(), FileType::SourceFile);
            self.graph.add_edge(product, input, ());
        }
    }

    // fn to_spdx(self) {
    //     let mut doc = self.doc;
    //     petgraph::algo::
    // }
}

struct SpdxGenerationOptions {
    build_dir: PathBuf,
    ignore: HashSet<String>,
}

impl SpdxGenerationOptions {
    fn perform_query(&self, target: &str) -> Result<QueryResult, anyhow::Error> {
        let output = Command::new("ninja")
            .arg("-t")
            .arg("query")
            .arg(target)
            .current_dir(&self.build_dir)
            .output()?;
        let stdout: String = String::from_utf8(output.stdout)?;

        let result = QueryResult::try_from_string(&stdout, target)
            .map_err(|e| anyhow::anyhow!("Query parsing error: {}", e.to_string()))?;
        Ok(result)
    }

    fn get_deps(&self) -> Result<Vec<DepsForOneFile>, anyhow::Error> {
        let output = Command::new("ninja")
            .arg("-t")
            .arg("deps")
            .current_dir(&self.build_dir)
            .output()?;
        let stdout: String = String::from_utf8(output.stdout)?;

        let result = recognize_deps(&stdout).map_err(|e| e.to_owned())?.1;

            // .map_err(|e| anyhow::anyhow!("Query parsing error: {}", e.to_string()))?;
        Ok(result)
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut generator = SpdxGenerator::new("extracted", PathBuf::from("/home/ryan/src/openxr"));
    {
        let contents = fs::read_to_string("deps.txt")?;

        for deps in recognize_deps(&contents).map_err(|e| e.to_owned())?.1 {
            generator.add_deps(deps);
        }
    }

    Ok(())
}
