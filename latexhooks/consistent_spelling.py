#!/usr/bin/env python3
import argparse
import re
import sys
import typing as t
from dataclasses import dataclass

RED = "\u001b[31m"
GREEN = "\u001b[32m"
YELLOW = "\u001b[33m"
BLUE = "\u001b[34m"
MAGENTA = "\u001b[35m"
CYAN = "\u001b[36m"
COLOR_RESET = "\u001b[0m"
ID2COLOR = {0: RED, 1: GREEN, 2: YELLOW, 3: BLUE, 4: MAGENTA, 5: CYAN}


@dataclass
class Rule:
    """A set of string (specified by regex) which should be checked for uniqueness."""

    # The name of the rules. Used to label the results if any.
    name: str
    # A regex which matches all variants of a phrase.
    regex: t.Pattern[str]


@dataclass
class MatchResult:
    # 0-based
    line_number: int
    # start and end of match within line
    span: t.Tuple[int, int]
    # The string which matches in the line
    pattern: str
    # Full line with match, for context
    line: str
    # The file where the match happened
    file_name: str


def emph_rule(phrase: str) -> Rule:
    """
    Check if the phrase only ever appears with or without a surrounding \\emph{...}.

    For example, "et al." can be spelled like "\\emph{et al.}" or "et al.", but it should be
    consistent.
    """
    regex = r"(?:\\emph\{)?"
    regex += r"(?:" + re.escape(phrase) + r")"
    regex += r"(?:\})?"
    return Rule(name=phrase, regex=re.compile(regex))


def search(rules: t.List[Rule], files: t.List[t.IO[str]]) -> bool:
    matches: t.Dict[str, t.List[MatchResult]] = dict()
    for f in files:
        for line_number, line in enumerate(f):
            for rule in rules:
                for match in rule.regex.finditer(line):
                    res = MatchResult(
                        line_number=line_number,
                        span=match.span(),
                        pattern=match[0],
                        line=line,
                        file_name=f.name,
                    )
                    matches.setdefault(rule.name, list()).append(res)

    found_different_spellings = False
    for pattern, results in matches.items():
        # Check if there are different variants in spelling
        pattern2id = {p: i for i, p in enumerate(set(x.pattern for x in results))}
        if 1 != len(pattern2id):
            # Found multiple spellings
            found_different_spellings = True

            current_file = None

            print(f"Found different spellings for: {pattern}")
            for res in results:
                # Only print the filename once
                if res.file_name != current_file:
                    current_file = res.file_name
                    print(current_file)

                c = ID2COLOR[pattern2id[res.pattern]]
                # res.line contains a linebreak
                print(
                    f"{c} {res.line_number+1:> 5} |{COLOR_RESET} {res.line}{c}       |{COLOR_RESET} {' '*(res.span[0])}{c}{'^'*(res.span[1]-res.span[0])}{COLOR_RESET}"
                )
            print()

    return found_different_spellings


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--emph",
        action="append",
        metavar=("phrase"),
        help="Check if the phrase appears with and without a surrounding \\emph{..}",
        default=list(),
    )
    parser.add_argument(
        "--regex",
        action="append",
        metavar=("name=regex"),
        type=lambda x: x.split("=", 2),
        help="Match all occurences of regex",
        default=list(),
    )
    parser.add_argument(
        "files",
        metavar="FILE",
        type=open,
        nargs="+",
        help="List of filenames to search in",
    )
    args = parser.parse_args()

    rules: t.List[Rule] = list()
    rules += [emph_rule(phrase) for phrase in args.emph]
    rules += [Rule(name=name, regex=re.compile(r)) for name, r in args.regex]

    if len(rules) == 0:
        sys.exit("No rules specified. See --help for how to use them.")

    files = sorted(args.files, key=lambda f: f.name)
    found_different_spellings = search(rules, files)
    if found_different_spellings:
        sys.exit("Found different spellings")


if __name__ == "__main__":
    main()
