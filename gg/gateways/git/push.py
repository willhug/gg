from plumbum import FG

from gg.gateways.git import git_cmd


def git_push(branch, force=False):
    command = git_cmd['push']
    if force:
        command = command['-f']
    command = command['origin', branch]

    return (command)& FG
