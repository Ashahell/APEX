import json
from pathlib import Path

ast = json.loads(Path('graphify-out/ast.json').read_text())
merged = {'nodes': ast['nodes'], 'edges': ast['edges'], 'hyperedges': [], 'input_tokens': 0, 'output_tokens': 0}
Path('graphify-out/extract.json').write_text(json.dumps(merged, indent=2))
total = len(merged["nodes"])
edges = len(merged["edges"])
print("Merged: " + str(total) + " nodes, " + str(edges) + " edges")