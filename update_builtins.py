import json
from pathlib import Path
import re


if __name__ == "__main__":
    builtins_rs_file = Path(__name__).parent / "src/values/builtins.rs"
    builtins_rs_source = builtins_rs_file.read_text()
    builtin_func_patt = r"\"(?P<builtin_name>\w+)\" => Some\("
    builtin_funcs = []
    for line in builtins_rs_source.splitlines():
        match = re.match(builtin_func_patt, line.strip())
        if not match :
            continue
        builtin_funcs.append(match.group("builtin_name").strip())
    print("Found built-in functions:", builtin_funcs)
    builtins_regexp = "\\b(" + "|".join(builtin_funcs) + ")\\b"
    print("Regexp:", builtins_regexp)

    syntax_file = Path(__name__).parent / "vscode-extension/syntaxes/calculator.tmLanguage.json"
    syntax = json.loads(syntax_file.read_text())
    syntax["repository"]["builtin_functions"]["patterns"][0]["match"] = builtins_regexp
    syntax_file.write_text(json.dumps(syntax, indent=4))
    