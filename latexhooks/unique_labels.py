#!/usr/bin/env python3
import argparse
import re
import sys
import typing as t
from dataclasses import dataclass


@dataclass
class MatchResult:
    """Information about a regex match"""

    line_number: int
    """ Line with a match, 0-based """
    span: t.Tuple[int, int]
    """ Start and end of match within line """
    pattern: str
    """ The string which matches in the line """
    line: str
    """ Full line with match, for context """
    file_name: str
    """ The file where the match happened """


def search(files: t.List[t.IO[str]]) -> bool:
    # re.Regex(r"\\label\{(.*?)\}", "label")
    re_label = re.compile(r"\\label\{\s*([^\s\}]*?)\s*\}", re.MULTILINE)
    # Regex explained:
    # Part 1: (([^%]|[^\\](?:\\\\)*\\%)*[^\\](?:\\\\)*)?
    #     This matches the text before a comment
    #   Part 1a: ([^%]|[^\\](?:\\\\)*\\%)*
    #       This matches either non-% symbols or a % preceded by an odd number of escaping backslashes
    #   Part 1b: [^\\](?:\\\\)*
    #       This matches an (optional) even number of escaping backslashes before the %
    # Part 2: (%.*)?
    #     This matches the % and everything after (the part we ignore)
    re_no_comment = re.compile(r"^(([^%]|[^\\](?:\\\\)*\\%)*[^\\](?:\\\\)*)?(%.*)?$")

    matches: t.Dict[str, t.List[MatchResult]] = dict()
    for f in files:
        for line_number, line in enumerate(f):
            match = re_no_comment.match(line.rstrip())
            if match is None:
                line_code = line
            else:
                line_code = match.group(1) or ""
            for match in re_label.finditer(line_code):
                res = MatchResult(
                    line_number=line_number,
                    span=match.span(),
                    pattern=match[1],
                    line=line,
                    file_name=f.name,
                )
                matches.setdefault(match[1], list()).append(res)

    found_multiple_definitions = False
    for pattern, results in matches.items():
        if len(results) > 1:
            print(f"Found multiple definitions for label {pattern}")
            # Found multiple spellings
            found_multiple_definitions = True

            current_file = None
            for res in results:
                # Only print the filename once
                if res.file_name != current_file:
                    current_file = res.file_name
                    print(current_file)

                if not res.line.endswith("\n"):
                    res.line += "\n"
                print(
                    f" {res.line_number+1:> 5} | {res.line}       | {' '*(res.span[0])}{'^'*(res.span[1]-res.span[0])}"
                )
            print()

    return found_multiple_definitions


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--individual",
        action="store_true",
        help="Allow duplicate labels if they are in different files",
    )
    parser.add_argument(
        "files",
        metavar="FILE",
        type=lambda x: open(x, encoding="utf-8"),
        nargs="+",
        help="List of filenames to search in",
    )
    args = parser.parse_args()

    files = sorted(args.files, key=lambda f: f.name)
    if args.individual:
        found_multiple_definitions = any(search([f]) for f in files)
    else:
        found_multiple_definitions = search(files)
    if found_multiple_definitions:
        sys.exit("Found multiple definitions of the same label")


if __name__ == "__main__":
    main()
