// Copyright 2022-2023, Collabora, Ltd.
//
// SPDX-License-Identifier: BSL-1.0
//
// Author: Ryan Pavlik <ryan.pavlik@collabora.com>

/// Parsing the `ninja -t deps` output
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{line_ending, none_of, not_line_ending},
    combinator::{self, fail, recognize, value, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};

// src/conformance/conformance_test/CMakeFiles/conformance_test.dir/__/framework/bitmask_to_string.cpp.o: #deps 318, deps mtime 1675791486835771571 (STALE)
//     /home/ryan/src/openxr/src/conformance/framework/bitmask_to_string.cpp
//     /usr/include/stdc-predef.h
//     /home/ryan/src/openxr/src/conformance/framework/bitmask_to_string.h
//     /home/ryan/src/openxr/src/conformance/framework/conformance_framework.h

/// Consumes and drops it
fn output_line_suffix<'a>(input: &'a str) -> IResult<&'a str, ()> {
    value((), pair(tag(": #deps"), not_line_ending))(input)
}

///
fn recognize_line_is_output_line<'a>(line: &'a str) -> IResult<&'a str, &'a Path> {
    let pos = line.find(": #deps");
    // let comb = match pos {
    //     Some(pos) => terminated(take(pos), output_line_suffix)
    // .map(Path::new),
    //     None => fail,
    // };
    if let Some(pos) = pos {
        terminated(take(pos), output_line_suffix)
            .map(Path::new)
            .parse(line)
    } else {
        fail(line)
    }
}

/// Does not consume the newline
fn recognize_output_line<'a>(input: &'a str) -> IResult<&'a str, &'a Path> {
    not_line_ending
        .and_then(recognize_line_is_output_line)
        .parse(input)
}

fn recognize_dep<'a>(input: &'a str) -> IResult<&'a str, &'a Path> {
    let recognize_path =
        preceded(tag("    "), recognize(pair(none_of(" "), not_line_ending))).map(Path::new);
    not_line_ending.and_then(recognize_path).parse(input)
}

#[derive(Debug, Clone)]
pub struct DepsForOneFile<'a, X>
where
    X: ToOwned<Owned = PathBuf>,
{
    pub output: Cow<'a, X>,
    pub inputs: Vec<Cow<'a, X>>,
}

impl<X> DepsForOneFile<'_, X>
where
    X: ToOwned<Owned = PathBuf>,
{
    pub fn into_owned<'a>(self) -> DepsForOneFile<'a, X> {
        Self {
            output: self.output.clone(),
            inputs: self.inputs.into_iter().cloned().collect(),
        }
    }
}

fn recognize_deps_for_one_file<'a>(input: &'a str) -> IResult<&'a str, DepsForOneFile<'a>> {
    let output_line = terminated(recognize_output_line, line_ending);
    let deps = many0(terminated(recognize_dep, line_ending));
    pair(output_line, deps)
        .map(|(output, inputs)| DepsForOneFile {
            output: Cow::Borrowed(output),
            inputs: inputs.into_iter().map(|p| Cow::Borrowed(p)).collect(),
        })
        .parse(input)
}

pub fn recognize_deps<'a>(input: &'a str) -> IResult<&'a str, Vec<DepsForOneFile<'a>>> {
    separated_list0(line_ending, recognize_deps_for_one_file).parse(input)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use nom::combinator::all_consuming;
    use nom::Finish;

    use super::{output_line_suffix, recognize_dep, recognize_output_line};

    #[test]
    fn test_output_line_suffix() {
        let mut parser = all_consuming(output_line_suffix);

        assert!(
            parser(": #deps 318, deps mtime 1675791486835771571 (STALE)")
                .finish()
                .is_ok()
        );
        assert!(parser(" #deps 318, deps mtime 1675791486835771571 (STALE)")
            .finish()
            .is_err());
        assert!(parser(": #d").finish().is_err());
    }

    #[test]
    fn test_output_line() {
        let mut parser = all_consuming(recognize_output_line);
        assert!(
            parser("src/conformance/conformance_test/CMakeFiles/conformance_test.dir/__/framework/bitmask_to_string.cpp.o: #deps 318, deps mtime 1675791486835771571 (STALE)")
                .finish()
                .is_ok()
        );
        let asdf = parser("src/conformance/conformance_test/CMakeFiles/conformance_test.dir/__/framework/bitmask_to_string.cpp.o: #deps 318, deps mtime 1675791486835771571 (STALE)");
        assert_eq!(asdf.finish().unwrap().1
            , Path::new("src/conformance/conformance_test/CMakeFiles/conformance_test.dir/__/framework/bitmask_to_string.cpp.o")
        );
        assert!(parser("\nasdf: #deps 318, deps mtime").finish().is_err());
    }

    #[test]
    fn test_dep() {
        let mut parser = all_consuming(recognize_dep);
        assert!(parser(
            "    /home/ryan/src/openxr/src/conformance/framework/bitmask_to_string.cpp"
        )
        .finish()
        .is_ok());
        assert!(parser(
            "    /home/ryan/src/openxr/src/conformance/framework/bitmask_to_string.cpp\n"
        )
        .is_err());
        assert!(parser(
            "     /home/ryan/src/openxr/src/conformance/framework/bitmask_to_string.cpp\n"
        )
        .is_err());
    }
}
