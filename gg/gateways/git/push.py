from plumbum import FG

from gg.gateways.git import git_cmd


def git_push(branch, force=False) -> int:
    command = git_cmd['push']
    if force:
        command = command['-f']
    command = command['origin', branch]

    return (command)& FG

def git_push_multi(branches, force=False) -> int:
    command = git_cmd['push']
    if force:
        command = command['-f']
    command = command['origin']
    for branch in branches:
        command = command[branch]

    return (command)& FG
