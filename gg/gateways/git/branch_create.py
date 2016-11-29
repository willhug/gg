from plumbum import FG

from gg.gateways.git import git_cmd

def git_checkout_new_branch(branch_name):
    command = git_cmd['checkout', '-b', branch_name]
    return (command) & FG
