from gg.gateways.git.branch_delete import safe_git_delete_branch
from gg.gateways.git.branch_delete_remote import safe_git_delete_remote_branch
from gg.gateways.git.fetch import git_fetch
from gg.lib.branch_name import get_prefix_branch_name

def delete_force_local_branch(branch: str) -> int:
    prefix_branch = get_prefix_branch_name(branch)
    safe_git_delete_branch(branch)
    safe_git_delete_branch(prefix_branch)

    git_fetch()
    return 0

def delete_force_remote_branch(branch: str) -> int:
    prefix_branch = get_prefix_branch_name(branch)
    safe_git_delete_remote_branch('origin', branch)
    safe_git_delete_remote_branch('origin', prefix_branch)

    git_fetch()
    return 0
