from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.push import git_push
from gg.lib.branch_name import get_prefix_branch_name, get_next_branch


def push_to_remote(branch: str) -> int:
    # Push the local branches to the remote
    prefix_branch = get_prefix_branch_name(branch)
    git_checkout(prefix_branch)
    git_push(prefix_branch, force=True)

    #Push the main branch
    git_checkout(branch)
    git_push(branch, force=True)

    return 0
