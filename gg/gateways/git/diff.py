from plumbum import FG

from gg.gateways.git import git_cmd

def git_diff(ref: str) -> int:
    """get the diff off of the ref"""
    command = git_cmd['diff', ref]
    return (command) & FG
