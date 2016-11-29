from typing import List

from gg.gateways.git import git_cmd

def get_all_branches() -> List[str]:
    """Get all the branches in the current git repo"""
    raw_branch_names = git_cmd['branch']().split('\n')
    branch_names = []
    for raw_branch_name in raw_branch_names:
        branch_name = raw_branch_name.strip('* ')
        if branch_name == '':
            continue
        if branch_name == '(no branch)':
            continue
        branch_names.append(branch_name)
    return branch_names
