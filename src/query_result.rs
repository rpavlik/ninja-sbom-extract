// Copyright 2022-2023, Collabora, Ltd.
//
// SPDX-License-Identifier: BSL-1.0
//
// Author: Ryan Pavlik <ryan.pavlik@collabora.com>

/// Parsing the `ninja -t query` output
use std::path::Path;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending},
    combinator::all_consuming,
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, tuple},
    Finish, IResult, Parser,
};

pub enum QueryInput {
    Normal(String),
    Implicit(String),
    OrderOnly(String),
}

pub struct QueryResult {
    pub input_desc: String,
    pub input: Vec<QueryInput>,
    pub outputs: Vec<String>,
}

impl QueryResult {
    pub fn try_from_string<'a>(
        input: &'a str,
        query_target: &'a str,
    ) -> Result<Self, nom::error::Error<&'a str>> {
        let result = all_consuming(query_results(query_target))
            .parse(input)
            .finish()?
            .1;
        Ok(result)
    }

    pub fn phony(&self) -> bool {
        self.input_desc == "phony"
    }
}

fn parse_input_line<'a>(input: &'a str) -> IResult<&'a str, QueryInput> {
    let order_only =
        preceded(tag("|| "), not_line_ending).map(|s: &str| QueryInput::OrderOnly(s.to_owned()));
    let implicit =
        preceded(tag("| "), not_line_ending).map(|s: &str| QueryInput::OrderOnly(s.to_owned()));
    let normal = not_line_ending.map(|s: &str| QueryInput::Normal(s.to_owned()));

    preceded(tag("    "), alt((order_only, implicit, normal)))(input)
}

// fn parse_query_results<'a>(query_target: &'a str, input: &'a str) -> IResult<&'a str, QueryResult> {
fn query_results<'a>(
    query_target: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, QueryResult> {
    move |input: &'a str| {
        let first_line = tuple((tag(query_target), tag(":"), line_ending));
        let input_desc =
            delimited(tag("  input: "), not_line_ending, line_ending).map(ToOwned::to_owned);
        let outputs_header = tag("  outputs:");
        let output_line = preceded(tag("    "), not_line_ending).map(ToOwned::to_owned);
        tuple((
            preceded(first_line, input_desc),
            many0(parse_input_line),
            preceded(
                outputs_header,
                preceded(line_ending, separated_list0(line_ending, output_line)),
            ),
        ))
        .map(|(input_desc, input, outputs)| QueryResult {
            input_desc,
            input,
            outputs,
        })
        .parse(input)
    }
}
