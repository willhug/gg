from typing import Callable, List

from gg.gateways.git import git_cmd
from gg.gateways.git.commit_info import get_commit

GIT_HASH_LENGTH = 40

def get_graph_template(branch_names: List[str], numCommits: int) -> List[str]:
    """
    Runs a Git log with graph properties and only containing the commit ref for each commit
    this will allow any callsites to replace the commit refs with whatever information they want
    """
    cmd = git_cmd['log', '--color', '--graph', "--pretty=format:%H", '-%d' % numCommits]
    for branch_name in branch_names:
        cmd = cmd[branch_name]
    return cmd().split("\n")

def graph_replace_hash(graph_line: str, get_replacement_for_hash_func: Callable[[str], str]) -> str:
    """Replace hash takes a graph line and replaces each hash with the results from the passed in closure"""
    hash = graph_line.split(" ")[-1] # Should be guaranteed from results of git log

    # The line doesn't have a hash, ignore
    if len(hash) != GIT_HASH_LENGTH:
        return graph_line

    new_info = get_replacement_for_hash_func(hash)
    return graph_line.replace(hash, new_info)

def print_graph(branch_names: List[str], lines: int = 100):
    """Print out the branch names using the graph view"""
    graph_template = get_graph_template(branch_names, lines)

    def replace_closure(hash: str) -> str:
        commit = get_commit(hash)
        return "%s - %s <%s>" % (commit.get_short_hash(), commit.title, commit.author)

    for line in graph_template:
        # print(line)
        updated_line = graph_replace_hash(line, replace_closure)
        print(updated_line)
