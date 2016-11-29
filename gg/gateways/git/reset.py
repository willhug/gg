from plumbum import FG

from gg.gateways.git import git_cmd

def git_reset(new_ref: str, hard: bool = True):
    """Cherry pick a series of changes onto the current checkout"""
    cmd = git_cmd['reset']

    if hard:
        cmd = cmd['--hard']

    cmd = cmd[new_ref]

    (cmd) & FG
