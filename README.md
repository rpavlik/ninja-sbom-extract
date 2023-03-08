# Ninja SBoM Extract

<!--
Copyright 2023, Collabora, Ltd.
SPDX-License-Identifier: CC-BY-4.0
-->

Sure, you can generate an SPDX document from a source tree with something like
[reuse](https://reuse.software/). But the project is big and has conditional
compilation, so not every file in the repo is relevant. Plus, what about system
files? If only somebody knew all the files that become part of the output...

This is a **WIP** tool to extract that data from the Ninja build tool. Right now
it mainly just parses the output of some Ninja commands that dump some of this
data.

Feel free to carry this onward and build something useful with it. Of course, it
passes `reuse lint`.
