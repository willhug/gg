from plumbum import FG

from gg.gateways.git import git_cmd

def git_rebase_interactive(ref: str) -> int:
    """perform an interactive rebase onto the ref"""
    command = git_cmd['rebase', '-i', ref]
    return (command) & FG

def git_rebase_commit_order(ref: str) -> int:
    """perform an interactive rebase onto the ref"""
    command = git_cmd['rebase', ref, '--exec', 'git commit --amend --reset-author --no-edit']
    return (command) & FG
