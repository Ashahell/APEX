import json
from pathlib import Path
from graphify.extract import collect_files, extract

code_files = [Path('F:/Projects/APEX/core/router/tests/streaming_tinysse_tests.rs')]
result = extract(code_files)
Path('graphify-out/ast.json').write_text(json.dumps(result, indent=2))
print(f'AST: {len(result["nodes"])} nodes, {len(result["edges"])} edges')