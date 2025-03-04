# AI Assistant Guide

Welcome to the ai-rustdoc project. Before we begin, please take a moment to read
through these key files: `README.md` for project overview, `STYLE.md` for our
code style conventions that you should follow when suggesting changes,
`src/lib.rs` for the main implementation, and `justfile` for workflow commands.
Once you've read those, you'll be well-equipped to help with the project.

Currently, we are in the process of iteratively running the `print_hex_docs`
test, inspecting its outputs, and making changes to the library to improve the
markdown output. Please iterate on this using the `just local-ci` command
instead of running the test directly.

*Don't* try to read any `rustdoc.json` file. These files are too big to fit in
your context window. You are welcome to search within it, but generally you
shouldn't need to - relevant information should be provided in the test output.
