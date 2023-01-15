import argparse
import os

from pybtex.database import OrderedCaseInsensitiveDict, parse_file


def sort_library(
    library, banned_fields: set = {"abstract", "file", "keywords", "mendeley-tags"}
):
    original_keys = list(library.entries.keys())
    library.original_keys = original_keys

    sorted_entries = sorted(library.entries.items(), key=lambda x: x[1].key)
    trimmed_entries = [(k, drop_fields(v, banned_fields)) for k, v in sorted_entries]

    library.entries = OrderedCaseInsensitiveDict()
    library.add_entries(trimmed_entries)

    library.sorted_keys = list(library.entries.keys())


def check_reorder(library):
    if library.original_keys == library.sorted_keys:
        return False
    return True


def drop_fields(entry, banned_fields: set = {}):
    fields = set(entry.fields.keys())
    for k in fields.intersection(banned_fields):
        del entry.fields[k]
    return entry


def overwrite_library(library, filename):
    library.to_file(filename)


def main(argv=None):

    parser = argparse.ArgumentParser()
    parser.add_argument("filenames", nargs="*", help="Filenames to run")
    parser.add_argument(
        "--banned-fields",
        nargs="*",
        help="Fields to strip from bibitem entries",
        dest="banned",
        default=["abstract", "file", "keywords", "mendeley-tags"],
    )
    parser.add_argument(
        "--silent-overwrite", action="store_true", dest="silent", default=False
    )
    parser.add_argument(
        "--check-only", action="store_true", dest="check_only", default=False
    )
    args = parser.parse_args(argv)

    return_value = 0

    banned_fields = set(args.banned)

    for filename in args.filenames:

        library = parse_file(filename)
        sort_library(library, banned_fields=banned_fields)

        if check_reorder(library):
            if args.check_only:
                return_value = 1
            elif args.silent:
                overwrite_library(library, filename)
            else:
                return_value = 1
                overwrite_library(library, filename)
                print(f"FIXED: {os.path.abspath(filename)}")
    return return_value


if __name__ == "__main__":
    exit(main())
