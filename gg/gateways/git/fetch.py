from plumbum import FG

from gg.gateways.git import git_cmd

def git_fetch() -> int:
    """Delete a git branch"""
    command = git_cmd['fetch', '-p', 'origin', 'master']
    return (command) & FG
