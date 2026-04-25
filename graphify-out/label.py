import json
import sys
from pathlib import Path
from graphify.build import build_from_json
from graphify.cluster import score_all
from graphify.analyze import god_nodes, surprising_connections, suggest_questions
from graphify.report import generate

extraction = json.loads(Path('graphify-out/extract.json').read_text())
detection = json.loads(Path('graphify-out/detect.json').read_text())
analysis = json.loads(Path('graphify-out/analysis.json').read_text())

G = build_from_json(extraction)
communities = {int(k): v for k, v in analysis['communities'].items()}
cohesion = {int(k): v for k, v in analysis['cohesion'].items()}
tokens = {"input": 0, "output": 0}

labels = {0: "TinySSE Test Functions", 1: "TinySseTest Struct"}

questions = suggest_questions(G, communities, labels)
report = generate(G, communities, cohesion, labels, analysis['gods'], analysis['surprises'], detection, tokens, ".", suggested_questions=questions)
Path("GRAPH_REPORT.md").write_text(report)
labels_json = {str(k): v for k, v in labels.items()}
Path("graphify-out/labels.json").write_text(json.dumps(labels_json))
print("Report updated with community labels")