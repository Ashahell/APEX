import json
import sys
from pathlib import Path
from graphify.build import build_from_json
from graphify.cluster import cluster, score_all
from graphify.analyze import god_nodes, surprising_connections, suggest_questions
from graphify.report import generate
from graphify.export import to_json

extraction = json.loads(Path('graphify-out/extract.json').read_text())
detection = {"total_files": 1, "total_words": 344, "needs_graph": True, "warning": None, "files": {"code": [], "document": [], "paper": []}}

G = build_from_json(extraction)
communities = cluster(G)
cohesion = score_all(G, communities)
tokens = {"input": 0, "output": 0}
gods = god_nodes(G)
surprises = surprising_connections(G, communities)
labels = {cid: "Community " + str(cid) for cid in communities}

questions = suggest_questions(G, communities, labels)
report = generate(G, communities, cohesion, labels, gods, surprises, detection, tokens, ".", suggested_questions=questions)
Path("GRAPH_REPORT.md").write_text(report)
to_json(G, communities, "graphify-out/graph.json")

analysis = {
    "communities": {str(k): v for k, v in communities.items()},
    "cohesion": {str(k): v for k, v in cohesion.items()},
    "gods": gods,
    "surprises": surprises,
    "questions": questions,
}
Path("graphify-out/analysis.json").write_text(json.dumps(analysis, indent=2))
if G.number_of_nodes() == 0:
    print("ERROR: Graph is empty")
    raise SystemExit(1)
print("Graph: " + str(G.number_of_nodes()) + " nodes, " + str(G.number_of_edges()) + " edges, " + str(len(communities)) + " communities")